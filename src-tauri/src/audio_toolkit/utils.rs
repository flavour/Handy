/// Returns the appropriate CPAL host for the current platform.
/// On Linux, defaults to CPAL's default host (usually ALSA).
/// You can override via `HANDY_CPAL_HOST`:
/// - `default`
/// - `alsa`
pub fn get_cpal_host() -> cpal::Host {
    #[cfg(target_os = "linux")]
    {
        let requested = std::env::var("HANDY_CPAL_HOST").ok();
        let host = match requested.as_deref() {
            None | Some("default") => cpal::default_host(),
            Some("alsa") => {
                cpal::host_from_id(cpal::HostId::Alsa).unwrap_or_else(|_| cpal::default_host())
            }
            Some(other) => {
                log::warn!("Unknown HANDY_CPAL_HOST={:?}, using default", other);
                cpal::default_host()
            }
        };

        log::info!(
            "CPAL host selected: {:?} (HANDY_CPAL_HOST={:?})",
            host.id(),
            requested
        );
        host
    }
    #[cfg(not(target_os = "linux"))]
    {
        cpal::default_host()
    }
}
