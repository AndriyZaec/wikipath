use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Router, extract::Path, extract::State, routing::get};
use lol_html::{RewriteStrSettings, element, rewrite_str};
use reqwest::StatusCode;

struct FetchedArticle {
    canonical_page: String,
    html: String,
}

#[derive(Debug)]
enum FetchedArticleError {
    Request(reqwest::Error),
    InvalidTitleEncoding(std::str::Utf8Error),
}

impl From<reqwest::Error> for FetchedArticleError {
    fn from(error: reqwest::Error) -> Self {
        Self::Request(error)
    }
}

impl From<std::str::Utf8Error> for FetchedArticleError {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::InvalidTitleEncoding(error)
    }
}

impl std::fmt::Display for FetchedArticleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Request(error) => {
                write!(f, "Wikipedia request failed: {error}")
            }
            Self::InvalidTitleEncoding(error) => {
                write!(f, "Invalid Wikipedia title encoding: {error}")
            }
        }
    }
}

impl std::error::Error for FetchedArticleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::InvalidTitleEncoding(error) => Some(error),
        }
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn article(
    State(client): State<reqwest::Client>,
    Path(page): Path<String>,
) -> Result<Response, StatusCode> {
    let fetched = fetch_article(&client, &page).await.map_err(|error| {
        eprintln!("Failed to fetch article {page}: {error}");

        match error {
            FetchedArticleError::Request(error)
                if error.status() == Some(StatusCode::NOT_FOUND) =>
            {
                StatusCode::NOT_FOUND
            }
            _ => StatusCode::BAD_GATEWAY,
        }
    })?;

    if page != fetched.canonical_page {
        let mut target = reqwest::Url::parse("http://127.0.0.1:3000/article/")
            .expect("Invalid application URL");

        target
            .path_segments_mut()
            .expect("URL supports path segment")
            .pop_if_empty()
            .push(&fetched.canonical_page);

        return Ok(Redirect::temporary(target.as_str()).into_response());
    }

    let html = rewrite_links(&fetched.html, fetched.canonical_page.as_ref())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Html(html).into_response())
}

fn decode_title(encoded: &str) -> Result<String, FetchedArticleError> {
    Ok(percent_encoding::percent_decode_str(encoded)
        .decode_utf8()?
        .into_owned())
}

async fn fetch_article(
    client: &reqwest::Client,
    page: &str,
) -> Result<FetchedArticle, FetchedArticleError> {
    let mut url =
        reqwest::Url::parse("https://en.wikipedia.org/w/rest.php/v1/page/")
            .expect("Invalid Wikipedia url");

    url.path_segments_mut()
        .expect("Base URL supports path segments")
        .pop_if_empty()
        .push(page)
        .push("html");

    let response = client.get(url).send().await?.error_for_status()?;
    let canonical_page = response
        .url()
        .path_segments()
        .and_then(|mut segment| segment.nth_back(1))
        .expect("Wikipedia URL must contain a page title")
        .to_owned();

    println!("Canonical page name: {canonical_page}");

    let canonical_page = decode_title(&canonical_page)?;
    let html = response.text().await?;
    Ok(FetchedArticle {
        canonical_page,
        html,
    })
}

fn rewrite_links(
    html: &str,
    current_page: &str,
) -> Result<String, lol_html::errors::RewritingError> {
    let mut current_url = reqwest::Url::parse("http://127.0.0.1:3000/article/")
        .expect("Invalid application URL");

    current_url
        .path_segments_mut()
        .expect("URL supports path segments")
        .pop_if_empty()
        .push(current_page);

    rewrite_str(
        html,
        RewriteStrSettings::new().append_element_content_handler(element!(
            "a[href]",
            |el| {
                if let Some(href) = el.get_attribute("href") {
                    if let Some(fragment) = href.strip_prefix("#") {
                        let mut target = current_url.clone();
                        target.set_fragment(Some(fragment));
                        el.set_attribute("href", target.as_str())?;
                        return Ok(());
                    }
                    match local_article_link(&href) {
                        Some(local_href) => {
                            el.set_attribute("href", &local_href)?
                        }
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
    use crate::{FetchedArticleError, decode_title};

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

    #[test]
    fn decodes_encoded_title() {
        let title = decode_title("C%2B%2B").expect("Valid title should decode");
        assert_eq!(title, "C++");
    }

    #[test]
    fn reject_invalid_utf8_title() {
        let result = decode_title("%FF");

        assert!(matches!(
            result,
            Err(FetchedArticleError::InvalidTitleEncoding(_))
        ));
    }
}
