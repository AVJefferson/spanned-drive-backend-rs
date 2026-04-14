use crate::clients::Clients;
use dashmap::DashMap;

pub struct AppState {
    pub clients: Clients,
    pub auth_tokens: DashMap<String, Vec<String>>,
}
