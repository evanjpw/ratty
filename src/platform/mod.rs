//! Platform-specific code is isolated in this module

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
pub mod bsd;

/// Common traits to be implemented by all platform-specific modules
pub trait PlatformInterface {
    fn initialize(&self) -> anyhow::Result<()>;
    fn cleanup(&self) -> anyhow::Result<()>;
    fn terminal_size(&self) -> (u16, u16);
}

/// Get the current platform interface
pub fn get_platform_interface() -> anyhow::Result<Box<dyn PlatformInterface>> {
    #[cfg(target_os = "macos")]
    return Ok(Box::new(macos::MacOSPlatform::new()));
    
    #[cfg(target_os = "linux")]
    return Ok(Box::new(linux::LinuxPlatform::new()));
    
    #[cfg(target_os = "windows")]
    return Ok(Box::new(windows::WindowsPlatform::new()));
    
    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    return Ok(Box::new(bsd::BSDPlatform::new()));
    
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows", target_os = "freebsd", target_os = "netbsd")))]
    anyhow::bail!("Unsupported platform")
}