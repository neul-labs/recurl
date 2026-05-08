//! JavaScript stealth patches for headless Chrome
//!
//! These patches help evade bot detection by modifying JavaScript properties
//! that are commonly used to detect headless browsers.
//!
//! Based on puppeteer-extra-plugin-stealth techniques.

/// JavaScript code to patch navigator.webdriver
pub const PATCH_WEBDRIVER: &str = r#"
Object.defineProperty(navigator, 'webdriver', {
    get: () => undefined,
    configurable: true
});
"#;

/// JavaScript code to patch navigator.plugins (make it non-empty)
pub const PATCH_PLUGINS: &str = r#"
Object.defineProperty(navigator, 'plugins', {
    get: () => {
        const plugins = [
            { name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer', description: 'Portable Document Format' },
            { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai', description: '' },
            { name: 'Native Client', filename: 'internal-nacl-plugin', description: '' }
        ];
        plugins.item = (index) => plugins[index] || null;
        plugins.namedItem = (name) => plugins.find(p => p.name === name) || null;
        plugins.refresh = () => {};
        return plugins;
    },
    configurable: true
});
"#;

/// JavaScript code to patch navigator.languages
pub const PATCH_LANGUAGES: &str = r#"
Object.defineProperty(navigator, 'languages', {
    get: () => ['en-US', 'en'],
    configurable: true
});
"#;

/// JavaScript code to patch chrome runtime
pub const PATCH_CHROME_RUNTIME: &str = r#"
window.chrome = {
    runtime: {
        connect: () => {},
        sendMessage: () => {},
        onMessage: { addListener: () => {} }
    },
    loadTimes: function() {
        return {
            requestTime: Date.now() / 1000,
            startLoadTime: Date.now() / 1000,
            commitLoadTime: Date.now() / 1000,
            finishDocumentLoadTime: Date.now() / 1000,
            finishLoadTime: Date.now() / 1000
        };
    },
    csi: function() { return {}; }
};
"#;

/// JavaScript code to patch permissions API
pub const PATCH_PERMISSIONS: &str = r#"
const originalQuery = window.navigator.permissions.query;
window.navigator.permissions.query = (parameters) => (
    parameters.name === 'notifications' ?
        Promise.resolve({ state: Notification.permission }) :
        originalQuery(parameters)
);
"#;

/// JavaScript code to hide iframe detection
pub const PATCH_IFRAME: &str = r#"
Object.defineProperty(HTMLIFrameElement.prototype, 'contentWindow', {
    get: function() {
        return this._contentWindow || window;
    }
});
"#;

/// JavaScript code to patch WebGL vendor/renderer
pub const PATCH_WEBGL: &str = r#"
const getParameterProto = WebGLRenderingContext.prototype.getParameter;
WebGLRenderingContext.prototype.getParameter = function(parameter) {
    if (parameter === 37445) {
        return 'Intel Inc.';
    }
    if (parameter === 37446) {
        return 'Intel Iris OpenGL Engine';
    }
    return getParameterProto.call(this, parameter);
};
"#;

/// JavaScript code to patch console.debug
pub const PATCH_CONSOLE: &str = r#"
window.console.debug = () => null;
"#;

/// JavaScript code to fix broken iframe contentWindow access
pub const PATCH_BROKEN_IMAGE: &str = r#"
['height', 'width'].forEach(property => {
    const imageDescriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, property);
    Object.defineProperty(HTMLImageElement.prototype, property, {
        ...imageDescriptor,
        get: function() {
            if (this.complete && this.naturalHeight == 0) {
                return 20;
            }
            return imageDescriptor.get.apply(this);
        },
    });
});
"#;

/// Get all stealth patches as a single JavaScript string
pub fn get_all_patches() -> String {
    [
        PATCH_WEBDRIVER,
        PATCH_PLUGINS,
        PATCH_LANGUAGES,
        PATCH_CHROME_RUNTIME,
        PATCH_PERMISSIONS,
        PATCH_WEBGL,
        PATCH_CONSOLE,
        PATCH_BROKEN_IMAGE,
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patches_not_empty() {
        assert!(!PATCH_WEBDRIVER.is_empty());
        assert!(!PATCH_PLUGINS.is_empty());
        assert!(!get_all_patches().is_empty());
    }

    #[test]
    fn test_all_patches_combined() {
        let combined = get_all_patches();
        assert!(combined.contains("navigator"));
        assert!(combined.contains("webdriver"));
        assert!(combined.contains("plugins"));
    }
}
