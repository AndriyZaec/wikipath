use axum::{extract::State, routing::get, Router};
use reqwest::StatusCode;

async fn health() -> &'static str {
    "ok"
}

async fn article(State(client): State<reqwest::Client>) -> Result<String, StatusCode> {
    fetch_article(&client, "Rust_(programming_language)")
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

async fn fetch_article(client: &reqwest::Client, page: &str) -> Result<String, reqwest::Error> {
    let url = format!("https://en.wikipedia.org/w/rest.php/v1/page/{page}/html");
    let response = client.get(&url).send().await?.error_for_status()?;

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
        .route("/article", get(article))
        .with_state(client.clone());

    let html = fetch_article(&client, "Rust_(programming_language)")
        .await
        .expect("Failed to fetch article");
    println!("Listening on http://127.0.0.1:3000");
    println!("Recived {} bytes of HTML", html.len());

    axum::serve(listener, app).await.expect("Server failed");
}
