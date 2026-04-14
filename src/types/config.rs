use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerCredentials {
    pub bitget: Option<BitgetCredentials>,
    pub metatrader: Option<MetaTraderCredentials>,
    pub xm: Option<MetaTraderCredentials>, // XM often uses MT4/MT5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTraderCredentials {
    pub login: String,
    pub password: String,
    pub server: String,
    pub api_token: Option<String>, // If using a service like MetaApi
}

impl BrokerCredentials {
    pub fn empty() -> Self {
        Self {
            bitget: None,
            metatrader: None,
            xm: None,
        }
    }

    pub fn load_from_env() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = dotenvy::dotenv();

            let bitget = std::env::var("BITGET_API_KEY").ok().map(|key| BitgetCredentials {
                api_key: key,
                api_secret: std::env::var("BITGET_API_SECRET").unwrap_or_default(),
                passphrase: std::env::var("BITGET_PASSPHRASE").unwrap_or_default(),
            });

            let metatrader = std::env::var("METATRADER_LOGIN").ok().map(|login| MetaTraderCredentials {
                login,
                password: std::env::var("METATRADER_PASSWORD").unwrap_or_default(),
                server: std::env::var("METATRADER_SERVER").unwrap_or_default(),
                api_token: std::env::var("METATRADER_API_TOKEN").ok(),
            });

            let xm = std::env::var("XM_LOGIN").ok().map(|login| MetaTraderCredentials {
                login,
                password: std::env::var("XM_PASSWORD").unwrap_or_default(),
                server: std::env::var("XM_SERVER").unwrap_or_default(),
                api_token: None,
            });

            Self {
                bitget,
                metatrader,
                xm,
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            // On WASM (browser), we cannot read environment variables directly.
            // Keys should be passed through JS or fetched via a secure vault.
            Self::empty()
        }
    }
}
