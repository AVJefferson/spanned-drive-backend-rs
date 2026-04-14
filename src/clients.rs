#[derive(Clone)]
pub struct Clients {}

impl Clients {
    pub async fn new_from_env_variables() -> Self {
        Self {}
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self {}
    }
}
