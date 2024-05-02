use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub l1_rpc_http: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            l1_rpc_http: "http://localhost:8545".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().expect(".env not found");
        Self {
            l1_rpc_http: env::var("L1_RPC_HTTP").unwrap_or("http://localhost:8545".to_string()),
        }
    }
}
