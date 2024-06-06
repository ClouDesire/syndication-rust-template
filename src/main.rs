use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use cloudesire_client::{DeploymentStatus, Subscription};
use log::{debug, info};
use serde::Deserialize;

pub mod cloudesire_client;

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

    if event.entity.ne("Subscription") {
        debug!("Skipping {} events", event.entity);
        return HttpResponse::NoContent();
    }

    let subscription = cloudesire_client::get_subscription(event.id);

    match event.lifecycle {
        Lifecycle::Created | Lifecycle::Modified => subscription_deploy(subscription),
        Lifecycle::Deleted => subscription_undeploy(subscription),
    }

    HttpResponse::NoContent()
}

fn subscription_deploy(subscription: Subscription) {
    match subscription.deployment_status {
        DeploymentStatus::Pending => {
            if subscription.paid {
                info!("Provision tenant resources");
                cloudesire_client::update_status(subscription.id, DeploymentStatus::Deployed);
            }
        }
        DeploymentStatus::Stopped => info!("Temporarily suspend the subscription"),
        DeploymentStatus::Deployed => info!("Check if tenant is OK"),
        _ => debug!("Unimplemented"),
    }
}

fn subscription_undeploy(subscription: Subscription) {
    info!("Unprovision tenant and release resources");
    cloudesire_client::update_status(subscription.id, DeploymentStatus::Undeployed);
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
