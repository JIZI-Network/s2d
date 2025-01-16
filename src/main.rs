mod config;
mod models;

use crate::config::ConfigRoot;
use actix_multipart::Multipart;
use actix_web::web::{Bytes, BytesMut};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use confy::ConfyError;
use env_logger::{Builder, Target};
use futures_util::StreamExt as _;
use log::{debug, info, trace, LevelFilter};
use serenity::builder::{CreateAttachment, CreateEmbed, ExecuteWebhook};
use serenity::http::Http;
use serenity::model::webhook::Webhook;
use std::collections::HashMap;
use std::str::FromStr;
use crate::models::Envelope;

const HELLO: [&str; 4] = [
    "       ___     __",
    "   ___ |_  |___/ /",
    "  (_-</ __// _  /Sendgrid to Discord",
    " /___/____/\\_,_/(c) 2025 JIZI All Rights Reserved.  ",
];

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = init_config().unwrap();
    // Initialize logger
    init_logger(&config);
    for hello in &HELLO {
        info!("{}", hello);
    }
    let file = confy::get_configuration_file_path("s2d", None);
    info!("The configuration path is {}", file.unwrap().display());

    info!(
        "Starting server at {}:{}",
        config.server.host, config.server.port
    );
    listen(&config).await
}

fn init_config() -> Result<ConfigRoot, ConfyError> {
    let config = confy::load("s2d", None);
    config
}

/// Initialize logger
fn init_logger(config: &ConfigRoot) {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.filter_level(LevelFilter::from_str(config.logger.level.as_str()).unwrap());
    builder.init();
}

/// Listen
async fn listen(_config: &ConfigRoot) -> std::io::Result<()> {
    let config = web::Data::new(_config.clone());
    let host = config.server.host.clone();
    let port = config.server.port.clone();
    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .route("/transfer", web::post().to(transfer))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}

fn bad_request(message: &str) -> HttpResponse {
    trace!("Bad request: {}", message);
    HttpResponse::BadRequest().body(message.to_string())
}

async fn transfer(
    config: web::Data<ConfigRoot>,
    query: web::Query<models::QueryParameters>,
    mut body: Multipart,
) -> impl Responder {
    let passphrase = &config.server.passphrase;
    if &query.passphrase != passphrase {
        return HttpResponse::BadRequest().body("Invalid passphrase");
    }

    debug!("Reading POST request...");

    let mut form_data: HashMap<String, String> = HashMap::from([]);
    // (filename, bytes)
    let mut files = Vec::<(String, Bytes)>::new();

    while let Some(item) = body.next().await {
        let mut field = item.unwrap();

        match field.content_type() {
            Some(content_type) => {
                if content_type.type_() == mime::APPLICATION
                    && content_type.subtype() == mime::OCTET_STREAM
                {
                    trace!("Received multipart file");
                    let content_disposition = match field.content_disposition() {
                        None => {
                            return bad_request("Missing content disposition")
                        }
                        Some(content_disposition) => content_disposition,
                    };
                    let filename = match content_disposition.get_filename() {
                        None => return bad_request("Missing filename"),
                        Some(filename) => filename.to_string(),
                    };
                    let mut bytes = BytesMut::new();
                    while let Some(chunk) = field.next().await {
                        let chunk = match chunk {
                            Ok(chunk) => chunk,
                            Err(_) => {
                                return bad_request("Failed to read chunk")
                            }
                        };
                        bytes.extend_from_slice(chunk.clone().as_ref());
                    }
                    files.push((filename.clone(), bytes.freeze()));
                }
            }
            None => {
                // form-data
                let name = match field.name() {
                    Some(name) => name.to_string(),
                    None => return bad_request("Missing name"),
                };

                let mut bytes = BytesMut::new();
                while let Some(chunk) = field.next().await {
                    let chunk = match chunk {
                        Ok(chunk) => chunk,
                        Err(_) => return bad_request("Failed to read chunk"),
                    };
                    bytes.extend_from_slice(chunk.clone().as_ref());
                }
                let data = match std::str::from_utf8(bytes.as_ref()) {
                    Ok(data) => data,
                    Err(_) => return bad_request("Failed to read data"),
                };
                form_data.insert(name.clone(), data.to_string());
            }
        }
    }
    trace!("file_data: {:?}", files);
    trace!("form_data: {:?}", form_data);

    debug!("Sending webhook request...");
    let http = Http::new("");


    let envelope = match serde_json::from_str::<Envelope>(form_data["envelope"].as_str()) {
        Ok(envelope) => envelope,
        Err(_) => return bad_request("Failed to parse envelope"),
    };
    for to in envelope.to {
        let webhook_url = match config.webhook.url.get(to.as_str()) {
            Some(url) => url,
            None => return bad_request("Missing to"),
        };
        let webhook = match Webhook::from_url(&http, webhook_url).await {
            Ok(webhook) => webhook,
            Err(_) => return bad_request("Invalid webhook"),
        };

        let builder = ExecuteWebhook::new()
            .username(config.webhook.username.clone())
            .avatar_url(config.webhook.avatar_url.clone())
            .embed(
                CreateEmbed::new()
                    .title(config.webhook.title.clone())
                    .field(config.webhook.from.clone(), form_data["from"].clone(), true)
                    .field(config.webhook.to.clone(), form_data["to"].clone(), true)
                    .field(
                        config.webhook.subject.clone(),
                        form_data["subject"].clone(),
                        true,
                    )
                    .field(
                        config.webhook.text.clone(),
                        form_data["text"].clone(),
                        false,
                    ),
            )
            .add_files(
                files
                    .iter()
                    .map(|(name, bytes)| CreateAttachment::bytes(bytes.to_vec(), name.clone())),
            );

        match webhook.execute(&http, false, builder).await  {
            Err(_) => return bad_request("Failed to execute webhook"),
            Ok(_) => {}
        }
    }
    HttpResponse::Ok().into()
}
