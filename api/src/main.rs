use actix_web::{middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder, dev::ServiceRequest, dev::ServiceResponse, Error, HttpRequest};
use serde::Deserialize;
use std::sync::Arc;

mod config;
mod kafka;
mod auth;

use config::Settings;
use kafka::KafkaProducer;

#[derive(Deserialize)]
struct Payload(serde_json::Value);

#[post("/ingest")]
async fn ingest(
    req: HttpRequest,
    body: web::Json<Payload>,
    producer: web::Data<Arc<KafkaProducer>>,
) -> impl Responder {
    // authorize request using JWT
    if let Err(resp) = auth::authorize_request(&req) {
        return resp;
    }
    // authorization: validate JWT from Authorization header
    // Note: we get HttpRequest via extractor if needed; but here we require the handler
    // to be called with a request and use a manual extractor instead. For simplicity,
    // leave this check to the caller scope instead.
    // serialize the inner serde_json::Value (Payload is a tuple struct)
    let payload_str = serde_json::to_string(&body.0 .0).unwrap_or_default();

    if let Err(e) = producer.send_nonblocking(payload_str) {
        tracing::error!(error = ?e, "enqueue to kafka failed");
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("{}", e)}));
    }

    HttpResponse::Ok().json(serde_json::json!({"status":"queued"}))
}

#[derive(Deserialize)]
struct AuthRequest { email: String, password: String }

#[post("/signup")]
async fn signup(body: web::Json<AuthRequest>) -> impl Responder {
    auth::signup(web::Json(auth::AuthRequest { email: body.email.clone(), password: body.password.clone() })).await
}

#[post("/login")]
async fn login(body: web::Json<AuthRequest>) -> impl Responder {
    auth::login(web::Json(auth::AuthRequest { email: body.email.clone(), password: body.password.clone() })).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load config
    let settings = Settings::from_file("config.yaml").expect("failed to load config");

    // initialize tracing subscriber using config.logging.level if present, else from env
    if let Some(logging) = &settings.logging {
        if let Some(level) = &logging.level {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::new(level))
                .init();
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                .init();
        }
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    let kafka_producer = KafkaProducer::new(&settings.kafka).expect("failed to create producer");

    let shared_prod = Arc::new(kafka_producer);

    let bind = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!(%bind, "starting server");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(settings.clone()))
            .app_data(web::Data::new(shared_prod.clone()))
            .service(signup)
            .service(login)
            .service(ingest)
    })
    .bind(bind)?
    .run()
    .await
}
