use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum DeploymentStatus {
    Pending,
    Deployed,
    Stopped,
    Undeployed,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: u32,
    pub deployment_status: DeploymentStatus,
    pub paid: bool,
}

pub fn get_subscription(id: u32) -> Subscription {
    let base_url = env::var("CMW_BASE_URL").unwrap_or("http://localhost:8081".to_string());
    let auth_token = env::var("CMW_AUTH_TOKEN").unwrap_or("test-token".to_string());

    let url = base_url + "/subscription/" + &id.to_string();
    ureq::get(&url)
        .set("CMW-Auth-Token", &auth_token)
        .call()
        .unwrap()
        .into_json()
        .unwrap()
}

pub fn update_status(subscription_id: u32, status: DeploymentStatus) {
    use log::info;

    info!("Setting subscription {} to {:?}", subscription_id, status);

    if env::var("CMW_READ_ONLY").is_ok() {
        return;
    }

    let base_url = env::var("CMW_BASE_URL").unwrap_or("http://localhost:8081".to_string());
    let auth_token = env::var("CMW_AUTH_TOKEN").unwrap_or("test-token".to_string());

    let url = base_url + "/subscription/" + &subscription_id.to_string();
    ureq::patch(&url)
        .set("CMW-Auth-Token", &auth_token)
        .send_json(ureq::json!({"deploymentStatus": status}))
        .unwrap();
}
