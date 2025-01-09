mod models;
mod config;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use env_logger::{Builder, Target};
use log::{error, info, LevelFilter};
use std::str::FromStr;
use confy::ConfyError;
use webhook;
use crate::config::ConfigRoot;

const HELLO: [&str; 4] = [
    "       ___     __",
    "   ___ |_  |___/ /",
    "  (_-</ __// _  /Sendgrid to Discord",
    " /___/____/\\_,_/(c) 2025 JIZI All Rights Reserved.  "
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

    info!("Starting server at {}:{}", config.server.host, config.server.port);
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

async fn transfer(
    config: web::Data<ConfigRoot>,
    query: web::Query<models::QueryParameters>,
    body: web::Json<models::Body>,
) -> impl Responder {
    let passphrase = &config.server.passphrase;
    if (&query.passphrase != passphrase) {
        return HttpResponse::BadRequest().body("Invalid passphrase");
    }

    let from = &body.from;
    let to = &body.to;
    let subject = &body.subject;
    let text = &body.text;
    let attachments = &body.attachments;

    if(!config.webhook.url.contains_key(to)) {
        return HttpResponse::BadRequest().body("The email address is not registered.");
    }

    match webhook::client::WebhookClient::new(config.webhook.url[to].as_str())
        .send(|message| {
            message
                .username(config.webhook.username.as_str())
                .avatar_url(config.webhook.avatar_url.as_str())
                .embed(|embed| {
                    embed
                        .title(config.webhook.title.as_str())
                        .field(config.webhook.from.as_str(), from, true)
                        .field(config.webhook.to.as_str(), to, true)
                        .field(config.webhook.subject.as_str(), subject, true)
                        .field(config.webhook.text.as_str(), text, false)
                        .field(config.webhook.attachments.as_str(), attachments, true)
                })
        })
        .await
    {
        Ok(_) => {}
        Err(e) => {
            error!("{}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    HttpResponse::Ok().body("OK")
}
