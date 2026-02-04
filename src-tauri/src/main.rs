// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Some dependencies (e.g. enigo) use Xlib internally while others use XCB.
    // If multiple threads end up touching X11 without XInitThreads, XCB can abort with:
    // "XInitThreads has not been called" / `xcb_xlib_threads_sequence_lost`.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_some() {
            // Safety: XInitThreads is expected to be called once, before any other Xlib calls.
            unsafe {
                x11::xlib::XInitThreads();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/dri").exists()
            && std::env::var("WAYLAND_DISPLAY").is_err()
            && std::env::var("XDG_SESSION_TYPE").unwrap_or_default() == "x11"
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }

    handy_app_lib::run()
}
