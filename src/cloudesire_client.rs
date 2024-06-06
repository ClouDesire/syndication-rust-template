use serde::Deserialize;

#[derive(Deserialize, Debug)]
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
    Subscription {
        id,
        deployment_status: DeploymentStatus::Pending,
        paid: false,
    }
}

pub fn update_status(subscription_id: u32, status: DeploymentStatus) {
    use log::info;

    info!("Setting subscription {} to {:?}", subscription_id, status);
}
