use std::sync::LazyLock;
use std::time::Duration;
use reqwest::Client;
use reqwest::header::USER_AGENT;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .expect("client")
});

pub fn create_http_client() -> Client { HTTP_CLIENT.clone() }
