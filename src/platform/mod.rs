//! Platform utilities and OS integration.

use std::path::{Path, PathBuf};

/// Get the assets directory path
pub fn assets_dir() -> PathBuf {
    // Try to find assets folder relative to executable or current dir
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));
    
    if let Some(exe) = exe_dir {
        let assets = exe.join("assets");
        if assets.exists() {
            return assets;
        }
    }
    
    // Fallback to current directory
    PathBuf::from("assets")
}

/// Read a file from the assets directory
pub fn read_asset(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    let full_path = assets_dir().join(path);
    std::fs::read(full_path)
}

/// Read a text file from the assets directory
pub fn read_asset_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    let full_path = assets_dir().join(path);
    std::fs::read_to_string(full_path)
}

/// Get platform name
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "linux")]
    return "linux";
    #[cfg(target_arch = "wasm32")]
    return "web";
    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_arch = "wasm32"
    )))]
    return "unknown";
}

/// Check if running on desktop
pub fn is_desktop() -> bool {
    cfg!(any(target_os = "windows", target_os = "macos", target_os = "linux"))
}

/// Check if running on web
pub fn is_web() -> bool {
    cfg!(target_arch = "wasm32")
}
