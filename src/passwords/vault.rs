use super::crypto::{
    decrypt_bytes, derive_master_key, encrypt_bytes, from_base64, generate_salt, to_base64,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

const VERIFIER_PLAINTEXT: &[u8] = b"NIKA_PASSWORD_VAULT_OK";

/// Credencial individual cifrada almacenada en disco
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedCredential {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub password_ciphertext: String, // Base64
    pub nonce: String,               // Base64
    pub created_at: String,
    pub updated_at: String,
}

/// Credencial descifrada para enviar a la interfaz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedCredential {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub password: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Estructura raíz del archivo cifrado de la bóveda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultData {
    pub salt: String,           // Base64 (16 bytes)
    pub verifier: String,       // Base64 del VERIFIER_PLAINTEXT cifrado
    pub verifier_nonce: String, // Base64 (12 bytes)
    #[serde(default)]
    pub entries: Vec<EncryptedCredential>,
}

/// Gestor de la bóveda de contraseñas de Nika Browser
pub struct PasswordVault {
    file_path: PathBuf,
    unlocked_key: Rc<RefCell<Option<[u8; 32]>>>,
}

impl PasswordVault {
    pub fn new() -> Self {
        let dir = glib::user_config_dir().join("nika-browser");
        let nika_vault = dir.join("passwords_vault.json");
        Self::new_with_path(nika_vault)
    }

    pub fn new_with_path(file_path: PathBuf) -> Self {
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        Self {
            file_path,
            unlocked_key: Rc::new(RefCell::new(None)),
        }
    }

    /// Comprueba si la bóveda ya fue creada con una contraseña maestra
    pub fn is_configured(&self) -> bool {
        self.file_path.exists()
    }

    /// Comprueba si la bóveda está actualmente desbloqueada en memoria
    pub fn is_unlocked(&self) -> bool {
        self.unlocked_key.borrow().is_some()
    }

    /// Bloquea la bóveda liberando la clave de memoria
    pub fn lock(&self) {
        *self.unlocked_key.borrow_mut() = None;
    }

    /// Configura la contraseña maestra por primera vez
    pub fn setup_master_password(&self, master_password: &str) -> Result<(), String> {
        if master_password.trim().is_empty() {
            return Err("La contraseña maestra no puede estar vacía.".to_string());
        }

        let salt = generate_salt();
        let key = derive_master_key(master_password, &salt);

        let (verifier_cipher, verifier_nonce) = encrypt_bytes(&key, VERIFIER_PLAINTEXT)?;

        let vault_data = VaultData {
            salt: to_base64(&salt),
            verifier: to_base64(&verifier_cipher),
            verifier_nonce: to_base64(&verifier_nonce),
            entries: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&vault_data)
            .map_err(|e| format!("Error serializando bóveda: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Error guardando archivo de bóveda: {}", e))?;

        *self.unlocked_key.borrow_mut() = Some(key);
        Ok(())
    }

    /// Desbloquea la bóveda verificando la contraseña maestra ingresada
    pub fn unlock(&self, master_password: &str) -> Result<bool, String> {
        if !self.file_path.exists() {
            return Err("La bóveda aún no ha sido configurada.".to_string());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Error leyendo bóveda: {}", e))?;
        let vault_data: VaultData = serde_json::from_str(&content)
            .map_err(|e| format!("Error parseando archivo de bóveda: {}", e))?;

        let salt_bytes = from_base64(&vault_data.salt)?;
        let key = derive_master_key(master_password, &salt_bytes);

        let verifier_cipher = from_base64(&vault_data.verifier)?;
        let nonce_bytes = from_base64(&vault_data.verifier_nonce)?;
        if nonce_bytes.len() != 12 {
            return Err("Longitud de nonce inválida en la bóveda.".to_string());
        }
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(&nonce_bytes);

        match decrypt_bytes(&key, &verifier_cipher, &nonce_arr) {
            Ok(decrypted) if decrypted == VERIFIER_PLAINTEXT => {
                *self.unlocked_key.borrow_mut() = Some(key);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Cambia la contraseña maestra re-cifrando todas las entradas existentes
    pub fn change_master_password(&self, old_master: &str, new_master: &str) -> Result<(), String> {
        if new_master.trim().is_empty() {
            return Err("La nueva contraseña no puede estar vacía.".to_string());
        }

        // 1. Obtener entradas descifradas actuales
        let entries = if self.is_unlocked() {
            self.list_entries()?
        } else {
            let valid = self.unlock(old_master)?;
            if !valid {
                return Err("Contraseña maestra actual incorrecta.".to_string());
            }
            self.list_entries()?
        };

        // 2. Generar nueva sal y clave
        let new_salt = generate_salt();
        let new_key = derive_master_key(new_master, &new_salt);
        let (new_verifier_cipher, new_verifier_nonce) = encrypt_bytes(&new_key, VERIFIER_PLAINTEXT)?;

        // 3. Re-cifrar cada entrada con la nueva clave
        let mut new_entries = Vec::new();
        for entry in entries {
            let (ciphertext, nonce) = encrypt_bytes(&new_key, entry.password.as_bytes())?;
            new_entries.push(EncryptedCredential {
                id: entry.id,
                domain: entry.domain,
                username: entry.username,
                password_ciphertext: to_base64(&ciphertext),
                nonce: to_base64(&nonce),
                created_at: entry.created_at,
                updated_at: entry.updated_at,
            });
        }

        let new_vault_data = VaultData {
            salt: to_base64(&new_salt),
            verifier: to_base64(&new_verifier_cipher),
            verifier_nonce: to_base64(&new_verifier_nonce),
            entries: new_entries,
        };

        let json = serde_json::to_string_pretty(&new_vault_data)
            .map_err(|e| format!("Error al serializar: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Error guardando bóveda actualizada: {}", e))?;

        *self.unlocked_key.borrow_mut() = Some(new_key);
        Ok(())
    }

    /// Retorna la lista de todas las credenciales descifradas
    pub fn list_entries(&self) -> Result<Vec<DecryptedCredential>, String> {
        let key = match *self.unlocked_key.borrow() {
            Some(k) => k,
            None => return Err("Bóveda bloqueada. Ingresa la contraseña maestra para ver las credenciales.".to_string()),
        };

        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Error leyendo bóveda: {}", e))?;
        let vault_data: VaultData = serde_json::from_str(&content)
            .map_err(|e| format!("Error parseando bóveda: {}", e))?;

        let mut result = Vec::new();
        for enc in vault_data.entries {
            let cipher = match from_base64(&enc.password_ciphertext) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let nonce = match from_base64(&enc.nonce) {
                Ok(n) if n.len() == 12 => {
                    let mut arr = [0u8; 12];
                    arr.copy_from_slice(&n);
                    arr
                }
                _ => continue,
            };

            if let Ok(dec) = decrypt_bytes(&key, &cipher, &nonce) {
                if let Ok(pass_str) = String::from_utf8(dec) {
                    result.push(DecryptedCredential {
                        id: enc.id,
                        domain: enc.domain,
                        username: enc.username,
                        password: pass_str,
                        created_at: enc.created_at,
                        updated_at: enc.updated_at,
                    });
                }
            }
        }

        Ok(result)
    }

    /// Añade o actualiza una credencial
    pub fn add_or_update_entry(&self, domain: &str, username: &str, password: &str) -> Result<DecryptedCredential, String> {
        let key = match *self.unlocked_key.borrow() {
            Some(k) => k,
            None => return Err("Bóveda bloqueada. Desbloquea para guardar contraseñas.".to_string()),
        };

        let clean_domain = domain.trim().to_lowercase();
        let clean_user = username.trim().to_string();

        let mut vault_data = if self.file_path.exists() {
            let content = fs::read_to_string(&self.file_path)
                .map_err(|e| format!("Error leyendo bóveda: {}", e))?;
            serde_json::from_str::<VaultData>(&content)
                .map_err(|e| format!("Error parseando bóveda: {}", e))?
        } else {
            return Err("Bóveda no inicializada".to_string());
        };

        let now = glib::DateTime::now_local()
            .map(|dt| dt.format("%Y-%m-%d %H:%M").map(|s| s.to_string()).unwrap_or_default())
            .unwrap_or_else(|_| "2026-10-31".to_string());

        let (ciphertext, nonce) = encrypt_bytes(&key, password.as_bytes())?;

        // Comprobar si ya existe una entrada para este dominio y usuario
        let existing_idx = vault_data.entries.iter().position(|e| {
            e.domain.to_lowercase() == clean_domain && e.username == clean_user
        });

        let id = if let Some(idx) = existing_idx {
            let existing_id = vault_data.entries[idx].id.clone();
            vault_data.entries[idx].password_ciphertext = to_base64(&ciphertext);
            vault_data.entries[idx].nonce = to_base64(&nonce);
            vault_data.entries[idx].updated_at = now.clone();
            existing_id
        } else {
            let new_id = format!("{}-{}", clean_domain.replace('.', "-"), glib::random_int());
            vault_data.entries.push(EncryptedCredential {
                id: new_id.clone(),
                domain: clean_domain.clone(),
                username: clean_user.clone(),
                password_ciphertext: to_base64(&ciphertext),
                nonce: to_base64(&nonce),
                created_at: now.clone(),
                updated_at: now.clone(),
            });
            new_id
        };

        let json = serde_json::to_string_pretty(&vault_data)
            .map_err(|e| format!("Error serializando: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Error guardando: {}", e))?;

        Ok(DecryptedCredential {
            id,
            domain: clean_domain,
            username: clean_user,
            password: password.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Elimina una credencial por su ID
    pub fn delete_entry(&self, id: &str) -> Result<(), String> {
        if !self.is_unlocked() {
            return Err("Bóveda bloqueada".to_string());
        }

        if !self.file_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Error leyendo bóveda: {}", e))?;
        let mut vault_data: VaultData = serde_json::from_str(&content)
            .map_err(|e| format!("Error parseando bóveda: {}", e))?;

        vault_data.entries.retain(|e| e.id != id);

        let json = serde_json::to_string_pretty(&vault_data)
            .map_err(|e| format!("Error serializando: {}", e))?;
        fs::write(&self.file_path, json)
            .map_err(|e| format!("Error guardando: {}", e))?;

        Ok(())
    }

    /// Busca credenciales para un dominio específico (para autofill)
    pub fn find_for_domain(&self, domain: &str) -> Vec<DecryptedCredential> {
        let clean = domain.trim().to_lowercase();
        if let Ok(entries) = self.list_entries() {
            entries
                .into_iter()
                .filter(|e| clean.contains(&e.domain) || e.domain.contains(&clean))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Cuenta cuántas contraseñas hay guardadas (sin necesidad de desbloquear)
    pub fn count_entries(&self) -> usize {
        if !self.file_path.exists() {
            return 0;
        }
        if let Ok(content) = fs::read_to_string(&self.file_path) {
            if let Ok(data) = serde_json::from_str::<VaultData>(&content) {
                return data.entries.len();
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_full_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("nika_test_vault_{}", glib::random_int()));
        let vault_file = temp_dir.join("test_vault.json");
        let vault = PasswordVault::new_with_path(vault_file.clone());

        assert!(!vault.is_configured());
        assert!(!vault.is_unlocked());

        // Configurar contraseña maestra
        vault.setup_master_password("MasterPassword123!").expect("Setup failed");
        assert!(vault.is_configured());
        assert!(vault.is_unlocked());

        // Añadir credencial
        let entry1 = vault.add_or_update_entry("github.com", "developer", "ghp_secureToken99").expect("Add failed");
        assert_eq!(entry1.domain, "github.com");
        assert_eq!(entry1.username, "developer");
        assert_eq!(entry1.password, "ghp_secureToken99");
        assert_eq!(vault.count_entries(), 1);

        // Bloquear bóveda
        vault.lock();
        assert!(!vault.is_unlocked());
        assert!(vault.list_entries().is_err());

        // Intentar desbloquear con contraseña incorrecta
        let wrong_unlock = vault.unlock("WrongPassword").expect("Unlock call failed");
        assert!(!wrong_unlock);
        assert!(!vault.is_unlocked());

        // Desbloquear con contraseña correcta
        let correct_unlock = vault.unlock("MasterPassword123!").expect("Unlock call failed");
        assert!(correct_unlock);
        assert!(vault.is_unlocked());

        // Listar y verificar datos descifrados
        let list = vault.list_entries().expect("List failed");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].domain, "github.com");
        assert_eq!(list[0].username, "developer");
        assert_eq!(list[0].password, "ghp_secureToken99");

        // Cambiar contraseña maestra
        vault.change_master_password("MasterPassword123!", "NewMasterPass456!").expect("Change master pass failed");

        // Bloquear y re-verificar con nueva contraseña
        vault.lock();
        assert!(!vault.unlock("MasterPassword123!").unwrap());
        assert!(vault.unlock("NewMasterPass456!").unwrap());

        let list_after_change = vault.list_entries().expect("List failed after change");
        assert_eq!(list_after_change.len(), 1);
        assert_eq!(list_after_change[0].password, "ghp_secureToken99");

        // Eliminar credencial
        vault.delete_entry(&entry1.id).expect("Delete failed");
        assert_eq!(vault.count_entries(), 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
