use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum Lifecycle {
    Created,
    Modified,
    Deleted,
}

#[derive(Deserialize)]
struct EventNotification {
    entity: String,
    id: u32,
    #[serde(rename = "type")]
    lifecycle: Lifecycle,
}

#[post("/event")]
async fn event(event: web::Json<EventNotification>) -> impl Responder {
    info!(
        "Received notification for {} with id {} of type {:?}",
        event.entity, event.id, event.lifecycle
    );
    HttpResponse::NoContent()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use env_logger::Env;

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| App::new().service(event))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
