use crate::external_systems::ExternalClients;
use dashmap::DashMap;

pub struct AppState {
    pub clients: ExternalClients,
    pub auth_tokens: DashMap<String, Vec<String>>,
}
