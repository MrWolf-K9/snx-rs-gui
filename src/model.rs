use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected_since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelServiceRequest {
    Connect(TunnelParams),
    Disconnect,
    GetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelServiceResponse {
    Ok,
    Error(String),
    ConnectionStatus(ConnectionStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelParams {
    pub server_name: String,
    pub user_name: String,
    pub password: String,
    pub log_level: String,
    pub reauth: bool,
    pub search_domains: Vec<String>,
    pub default_route: bool,
    pub no_routing: bool,
    pub no_dns: bool,
    pub no_cert_check: bool,
    pub tunnel_type: TunnelType,
    pub ca_cert: Option<PathBuf>,
    pub login_type: LoginType,
}

impl Default for TunnelParams {
    fn default() -> Self {
        TunnelParams {
            server_name: String::new(),
            user_name: String::new(),
            password: String::new(),
            log_level: String::from("info"),
            reauth: true,
            search_domains: vec!["".to_string()],
            default_route: false,
            no_routing: false,
            no_dns: false,
            no_cert_check: false,
            tunnel_type: TunnelType::default(),
            ca_cert: None,
            login_type: LoginType::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub tunnel_params: TunnelParams,
    pub remember_me: bool,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum TunnelType {
    #[default]
    Ssl,
    Ipsec,
}

impl fmt::Display for TunnelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TunnelType::Ssl => write!(f, "SSL"),
            TunnelType::Ipsec => write!(f, "IPSec"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoginType {
    Password,
    PasswordWithMfa,
    #[default]
    PasswordWithMsAuth,
    EmergencyAccess,
    SsoAzure,
}
