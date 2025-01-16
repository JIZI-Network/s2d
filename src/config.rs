use rand::distributions::{Alphanumeric, DistString};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ConfigRoot {
    pub logger: ConfigLogger,
    pub webhook: ConfigWebhook,
    pub server: ConfigServer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigLogger {
    pub level: String,
}

impl Default for ConfigLogger {
    fn default() -> Self {
        Self {
            level: "Info".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigWebhook {
    pub url: HashMap<String, String>,
    pub username: String,
    pub avatar_url: String,
    pub title: String,
    pub from: String,
    pub to: String,
    pub subject: String,
    pub text: String,
    pub attachments: String,
    pub might_be_a_spam: String,
    pub might_not_be_a_spam: String,
    pub spam_score: String,
}

impl Default for ConfigWebhook {
    fn default() -> Self {
        Self {
            url: HashMap::from([(
                "example@example.com".to_string(),
                "https://discord.com/api/webhooks/...".to_string(),
            )]),
            username: "JIZI-Network メール転送".to_string(),
            avatar_url: "https://github.com/Jizi-Network.png".to_string(),
            title: "メールを受信しました".to_string(),
            from: "送信者".to_string(),
            to: "受信者".to_string(),
            subject: "件名".to_string(),
            text: "本文".to_string(),
            attachments: "添付ファイルの個数".to_string(),
            might_be_a_spam: "スパムメールの可能性が高いです。注意してください。".to_string(),
            might_not_be_a_spam: "スパムメールの可能性は低いです。".to_string(),
            spam_score: "spam_score".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigServer {
    pub host: String,
    pub port: u16,
    pub passphrase: String,
    pub required_spam_score: f64
}

impl Default for ConfigServer {
    fn default() -> Self {
        let passphrase = Alphanumeric
            .sample_string(&mut thread_rng(), 32)
            .to_string();
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            passphrase,
            required_spam_score: 5.0,
        }
    }
}
