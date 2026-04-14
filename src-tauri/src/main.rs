// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK on Wayland (Hyprland) fails EGL init unless DMABUF rendering is
    // disabled. Must be set before run() because WebKit initialises at startup.
    #[cfg(target_os = "linux")]
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    enhabitz_lib::run()
}
