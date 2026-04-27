mod allowed_clients;
mod compile_time_checks;
mod constants;
mod error;
mod external_systems;
mod logger;
mod middlewares;
mod routes;
mod schedules;
mod server;
mod state;
mod test_utils;
mod validation;

#[tokio::main]
async fn main() {
    schedules::init_scheduler().await;
    server::start_server().await;
}
