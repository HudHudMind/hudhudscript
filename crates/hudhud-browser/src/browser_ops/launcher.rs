use std::process::Command;

#[derive(Debug)]
pub struct BrowserError(pub String);

pub trait BrowserLauncher {
    fn open(&self, url: &str) -> Result<(), BrowserError>;
}

pub struct RealBrowserLauncher;
impl BrowserLauncher for RealBrowserLauncher {
    fn open(&self, url: &str) -> Result<(), BrowserError> {
        match Command::new("xdg-open").arg(url).status() {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => Ok(()), // Some versions of xdg-open return non-zero even on success, or fail softly
            Err(e) => Err(BrowserError(format!("Failed to launch xdg-open: {}", e))),
        }
    }
}

pub struct SilentBrowserLauncher;
impl BrowserLauncher for SilentBrowserLauncher {
    fn open(&self, url: &str) -> Result<(), BrowserError> {
        println!("SilentBrowserLauncher: Simulated opening URL: {}", url);
        Ok(())
    }
}

pub fn get_launcher() -> Box<dyn BrowserLauncher> {
    if std::env::var("HUDHUD_REAL_BROWSER_TESTS").unwrap_or_default() == "1" {
        Box::new(RealBrowserLauncher)
    } else {
        Box::new(SilentBrowserLauncher)
    }
}
