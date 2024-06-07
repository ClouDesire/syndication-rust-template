use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use cloudesire_client::{DeploymentStatus, Subscription};
use log::{debug, info};
use serde::{Deserialize, Serialize};

pub mod cloudesire_client;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
enum Lifecycle {
    Created,
    Modified,
    Deleted,
}

#[derive(Serialize, Deserialize)]
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
        _ => todo!(),
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

#[cfg(test)]
mod tests {
    use actix_web::{test, App};
    use std::env;

    use super::*;

    #[actix_web::test]
    async fn test_event_post() {
        let mut server = mockito::Server::new_async().await;

        let url = server.url();
        env::set_var("CMW_BASE_URL", url);
        env::remove_var("CMW_READ_ONLY");

        server
            .mock("GET", "/subscription/1")
            .with_body(r#"{"id": 1, "deploymentStatus": "PENDING", "paid": true}"#)
            .create_async()
            .await;

        let mock = server
            .mock("PATCH", "/subscription/1")
            .match_body(mockito::Matcher::JsonString(
                r#"{"deploymentStatus": "DEPLOYED"}"#.to_string(),
            ))
            .create_async()
            .await;

        let app = test::init_service(App::new().service(event)).await;
        let req = test::TestRequest::post()
            .uri("/event")
            .set_json(EventNotification {
                entity: "Subscription".to_string(),
                id: 1,
                lifecycle: Lifecycle::Created,
            })
            .to_request();
        let res = test::call_service(&app, req).await;

        mock.assert_async().await;
        assert!(res.status().is_success());
    }
}
