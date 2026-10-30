use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use lol_html::{element, rewrite_str, RewriteStrSettings};
use reqwest::StatusCode;

async fn health() -> &'static str {
    "ok"
}

async fn article(
    State(client): State<reqwest::Client>,
    Path(page): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let html = fetch_article(&client, &page).await.map_err(|error| {
        eprintln!("Failed to fetch article {page}: {error}");
        if error.status() == Some(StatusCode::NOT_FOUND) {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_GATEWAY
        }
    })?;
    let html = rewrite_links(&html).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html))
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

fn rewrite_links(html: &str) -> Result<String, lol_html::errors::RewritingError> {
    rewrite_str(
        html,
        RewriteStrSettings::new().append_element_content_handler(element!(
            "a[rel~=\"mw:WikiLink\"]",
            |el| {
                if let Some(href) = el.get_attribute("href") {
                    match local_article_link(&href) {
                        Some(local_href) => el.set_attribute("href", &local_href)?,
                        None => el.remove_attribute("href"),
                    }
                }
                Ok(())
            }
        )),
    )
}

fn local_article_link(href: &str) -> Option<String> {
    let prefixes = ["Category:", "File:", "Wikipedia:"];
    href.strip_prefix("./").and_then(|page| {
        let service_prefix = prefixes.iter().any(|p| page.starts_with(p));
        if service_prefix {
            None
        } else {
            Some(format!("http://127.0.0.1:3000/article/{page}"))
        }
    })
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

#[cfg(test)]
mod tests {
    use super::local_article_link;

    #[test]
    fn category_link_is_rejected() {
        assert_eq!(local_article_link("./Category:Stars"), None)
    }

    #[test]
    fn allowed_article_link() {
        assert_eq!(
            local_article_link("./Sun"),
            Some(String::from("http://127.0.0.1:3000/article/Sun"))
        );
    }

    #[test]
    fn external_link_rejected() {
        assert_eq!(local_article_link("https://example.com"), None);
    }
}
