use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteProtocol {
    Ssh,
    Ftp,
    Sftp,
}

impl std::fmt::Display for RemoteProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteProtocol::Ssh => write!(f, "SSH"),
            RemoteProtocol::Ftp => write!(f, "FTP"),
            RemoteProtocol::Sftp => write!(f, "SFTP"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteConfig {
    pub protocol: RemoteProtocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub passphrase: Option<String>,
}

impl RemoteConfig {
    pub fn new(protocol: RemoteProtocol, host: &str, port: u16, username: &str) -> Self {
        Self {
            protocol,
            host: host.to_string(),
            port,
            username: username.to_string(),
            password: None,
            private_key_path: None,
            passphrase: None,
        }
    }

    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    pub fn with_private_key(mut self, path: PathBuf, passphrase: Option<&str>) -> Self {
        self.private_key_path = Some(path);
        self.passphrase = passphrase.map(|p| p.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RemoteConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct RemoteFileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub permissions: u32,
}

impl RemoteFileInfo {
    pub fn new(name: &str, path: &str, is_directory: bool) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            is_directory,
            size: 0,
            permissions: 0o644,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteConnection {
    config: RemoteConfig,
    status: RemoteConnectionStatus,
    current_path: String,
}

impl RemoteConnection {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            config,
            status: RemoteConnectionStatus::Disconnected,
            current_path: "/".to_string(),
        }
    }

    pub fn config(&self) -> &RemoteConfig {
        &self.config
    }

    pub fn status(&self) -> &RemoteConnectionStatus {
        &self.status
    }

    pub fn current_path(&self) -> &str {
        &self.current_path
    }

    pub fn connect(&mut self) -> Result<(), String> {
        self.status = RemoteConnectionStatus::Connecting;
        
        // In real implementation, this would establish the connection
        // For now, we just simulate it
        self.status = RemoteConnectionStatus::Connected;
        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.status = RemoteConnectionStatus::Disconnected;
        self.current_path = "/".to_string();
    }

    pub fn is_connected(&self) -> bool {
        self.status == RemoteConnectionStatus::Connected
    }

    pub fn list_files(&self, _path: &str) -> Result<Vec<RemoteFileInfo>, String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }
        
        // In real implementation, this would list remote files
        Ok(vec![])
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }
        
        self.current_path = path.to_string();
        Ok(())
    }

    pub fn upload(&self, _local_path: &PathBuf, _remote_path: &str) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }
        
        // In real implementation, this would upload the file
        Ok(())
    }

    pub fn download(&self, _remote_path: &str, _local_path: &PathBuf) -> Result<(), String> {
        if !self.is_connected() {
            return Err("Not connected".to_string());
        }
        
        // In real implementation, this would download the file
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RemoteManager {
    connections: Vec<RemoteConnection>,
}

impl RemoteManager {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn connections(&self) -> &[RemoteConnection] {
        &self.connections
    }

    pub fn connections_mut(&mut self) -> &mut Vec<RemoteConnection> {
        &mut self.connections
    }

    pub fn add_connection(&mut self, config: RemoteConfig) -> usize {
        let conn = RemoteConnection::new(config);
        self.connections.push(conn);
        self.connections.len() - 1
    }

    pub fn remove_connection(&mut self, index: usize) -> bool {
        if index < self.connections.len() {
            self.connections.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_connection(&self, index: usize) -> Option<&RemoteConnection> {
        self.connections.get(index)
    }

    pub fn get_connection_mut(&mut self, index: usize) -> Option<&mut RemoteConnection> {
        self.connections.get_mut(index)
    }
}

impl Default for RemoteManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_protocol_display() {
        assert_eq!(RemoteProtocol::Ssh.to_string(), "SSH");
        assert_eq!(RemoteProtocol::Ftp.to_string(), "FTP");
        assert_eq!(RemoteProtocol::Sftp.to_string(), "SFTP");
    }

    #[test]
    fn test_remote_config_new() {
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        assert_eq!(config.protocol, RemoteProtocol::Ssh);
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "user");
    }

    #[test]
    fn test_remote_config_with_password() {
        let config = RemoteConfig::new(RemoteProtocol::Ftp, "example.com", 21, "user")
            .with_password("pass123");
        assert_eq!(config.password, Some("pass123".to_string()));
    }

    #[test]
    fn test_remote_connection_new() {
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        let conn = RemoteConnection::new(config);
        assert_eq!(*conn.status(), RemoteConnectionStatus::Disconnected);
        assert_eq!(conn.current_path(), "/");
    }

    #[test]
    fn test_remote_connection_connect() {
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        let mut conn = RemoteConnection::new(config);
        
        assert!(conn.connect().is_ok());
        assert!(conn.is_connected());
    }

    #[test]
    fn test_remote_connection_disconnect() {
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        let mut conn = RemoteConnection::new(config);
        
        conn.connect().unwrap();
        assert!(conn.is_connected());
        
        conn.disconnect();
        assert!(!conn.is_connected());
    }

    #[test]
    fn test_remote_manager_new() {
        let manager = RemoteManager::new();
        assert!(manager.connections().is_empty());
    }

    #[test]
    fn test_remote_manager_add_connection() {
        let mut manager = RemoteManager::new();
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        let index = manager.add_connection(config);
        
        assert_eq!(index, 0);
        assert_eq!(manager.connections().len(), 1);
    }

    #[test]
    fn test_remote_manager_remove_connection() {
        let mut manager = RemoteManager::new();
        let config = RemoteConfig::new(RemoteProtocol::Ssh, "example.com", 22, "user");
        manager.add_connection(config);
        
        assert!(manager.remove_connection(0));
        assert!(manager.connections().is_empty());
    }
}
