use webkit6::{UserContentInjectedFrames, UserContentManager, UserScript, UserScriptInjectionTime};

pub const INTERNAL_WORLD: &str = "NikaInternal";

const BRIDGE_DEFAULT_WORLD: &str = r#"
(function () {
  if (window.nikaSend) return;
  window.nikaSend = function (name, data) {
    try {
      document.dispatchEvent(new CustomEvent('nika-internal-message', {
        detail: JSON.stringify({
          name: String(name),
          data: data === null || data === undefined ? '' : String(data)
        })
      }));
    } catch (e) {}
  };
})();
"#;

const BRIDGE_INTERNAL_WORLD: &str = r#"
(function () {
  document.addEventListener('nika-internal-message', function (event) {
    if (location.protocol !== 'nika:') return;
    if (!window.webkit || !window.webkit.messageHandlers) return;

    try {
      var payload = JSON.parse(event.detail);
      var handler = window.webkit.messageHandlers[payload.name];
      if (handler && typeof handler.postMessage === 'function') {
        handler.postMessage(payload.data);
      }
    } catch (e) {}
  }, false);
})();
"#;

/// Instala el puente que solo permite a las páginas internas (nika://)
/// comunicarse con los handlers nativos registrados en `INTERNAL_WORLD`.
pub fn install_internal_bridge(content_manager: &UserContentManager) {
    let default_script = UserScript::new(
        BRIDGE_DEFAULT_WORLD,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    );
    content_manager.add_script(&default_script);

    let internal_script = UserScript::for_world(
        BRIDGE_INTERNAL_WORLD,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        INTERNAL_WORLD,
        &[],
        &[],
    );
    content_manager.add_script(&internal_script);
}
