use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SystemIntegration;

impl SystemIntegration {
    pub fn new() -> Self {
        Self
    }

    pub fn set_as_default_file_manager(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            self.set_windows_default()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err("Not supported on this platform".to_string())
        }
    }

    pub fn restore_default_file_manager(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            self.restore_windows_default()
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err("Not supported on this platform".to_string())
        }
    }

    pub fn is_default_file_manager(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            self.check_windows_default()
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    pub fn open_path(&self, path: &PathBuf) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open path: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open path: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open path: {}", e))?;
            Ok(())
        }
    }

    pub fn open_file(&self, path: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap_or("")])
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
            Ok(())
        }
    }

    pub fn reveal_in_explorer(&self, path: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .args(["/select,", path.to_str().unwrap_or("")])
                .spawn()
                .map_err(|e| format!("Failed to reveal in explorer: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .args(["-R", path.to_str().unwrap_or("")])
                .spawn()
                .map_err(|e| format!("Failed to reveal in finder: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            let parent = path.parent().unwrap_or(path);
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to reveal in file manager: {}", e))?;
            Ok(())
        }
    }

    pub fn copy_to_clipboard(&self, text: &str) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "echo", text, "|", "clip"])
                .spawn()
                .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("pbcopy")
                .arg(text)
                .spawn()
                .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .arg(text)
                .spawn()
                .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
            Ok(())
        }
    }

    #[cfg(target_os = "windows")]
    fn set_windows_default(&self) -> Result<(), String> {
        use std::process::Command;

        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?
            .to_string_lossy()
            .to_string();

        // Set as default for Explorer
        Command::new("reg")
            .args([
                "add",
                r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.open",
                "/ve",
                "/t",
                "REG_SZ",
                "/d",
                &format!("\"{}\" \"%1\"", exe_path),
                "/f",
            ])
            .output()
            .map_err(|e| format!("Failed to set registry: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn restore_windows_default(&self) -> Result<(), String> {
        use std::process::Command;

        Command::new("reg")
            .args([
                "delete",
                r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.open",
                "/f",
            ])
            .output()
            .map_err(|e| format!("Failed to restore registry: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn check_windows_default(&self) -> bool {
        use std::process::Command;

        let output = Command::new("reg")
            .args([
                "query",
                r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.open",
                "/ve",
            ])
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("zero_explorer")
            }
            Err(_) => false,
        }
    }
}

impl Default for SystemIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_integration_new() {
        let integration = SystemIntegration::new();
        assert!(!integration.is_default_file_manager());
    }

    #[test]
    fn test_system_integration_open_path() {
        let integration = SystemIntegration::new();
        // Just test that the function doesn't panic
        let _ = integration.open_path(&PathBuf::from("."));
    }
}
