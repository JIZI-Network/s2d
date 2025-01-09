use serde::Deserialize;

#[derive( Deserialize )]
pub struct Body {
    pub headers: String,
    pub dkim: String,
    pub to: String,
    pub text: String,
    pub html: Option<String>,
    pub from: String,
    pub sender_ip: String,
    pub spam_report: Option<String>,
    pub envelope: String,
    pub attachments: String,
    pub subject: String,
    pub spam_score: Option<u32>,
    #[serde(rename = "attachment-info")]
    pub attachment_info: Option<AttachmentInfo>,
    pub charsets: String,
    #[serde(rename = "SPF")]
    pub spf: String,
}

#[derive( Deserialize )]
pub struct AttachmentInfo {
    pub filename: String,
    #[serde(rename = "type")]
    pub _type: String,
    #[serde(rename = "content-id")]
    pub content_id: u32,
}

#[derive( Deserialize )]
pub struct QueryParameters {
    pub passphrase: String,
}