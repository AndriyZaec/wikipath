use axum::{routing::get, Router};

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind port 3000");

    let client = reqwest::Client::builder()
        .user_agent("WikiPath/0.1 (https://github.com/AndriyZaec)")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("Failed to create HTTP client");

    let response = client
        .get("https://en.wikipedia.org/w/rest.php/v1/page/Rust_(programming_language)/html")
        .send()
        .await
        .expect("Failed to request Wikipedia");

    let html = response
        .error_for_status()
        .expect("Wikipedia returned an error status")
        .text()
        .await
        .expect("Failed to read response body");
    println!("Listening on http://127.0.0.1:3000");
    println!("Recived {} bytes of HTML", html.len());

    axum::serve(listener, app).await.expect("Server failed");
}
