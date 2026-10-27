use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use reqwest::StatusCode;

async fn health() -> &'static str {
    "ok"
}

async fn article(
    State(client): State<reqwest::Client>,
    Path(page): Path<String>,
) -> Result<Html<String>, StatusCode> {
    fetch_article(&client, &page)
        .await
        .map(Html)
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn fetch_article(client: &reqwest::Client, page: &str) -> Result<String, reqwest::Error> {
    let mut url = reqwest::Url::parse("https://en.wikipedia.org/w/rest.php/v1/page/")
        .expect("Invalid Wikipedia url");

    url.path_segments_mut()
        .expect("Base URL supports path segments")
        .pop_if_empty()
        .push(page)
        .push("html");

    let response = client.get(url).send().await?.error_for_status()?;

    let html = response.text().await?;
    Ok(html)
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port 3000");

    let client = reqwest::Client::builder()
        .user_agent("WikiPath/0.1 (https://github.com/AndriyZaec)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client");

    let app = Router::new()
        .route("/health", get(health))
        .route("/article/{page}", get(article))
        .with_state(client);

    println!("Listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("Server failed");
}
