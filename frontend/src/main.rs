use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

enum HealthState {
    Loading,
    Ready(String),
    Failed(String),
}

enum ArticleState {
    Loading,
    Ready(String),
    Failed(String),
}

impl std::fmt::Display for HealthState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthState::Loading => write!(f, "Loading..."),
            HealthState::Ready(text) => write!(f, "{text}"),
            HealthState::Failed(error) => write!(f, "{error}"),
        }
    }
}

impl std::fmt::Display for ArticleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArticleState::Loading => write!(f, "Loading article..."),
            ArticleState::Ready(html) => {
                let len = html.len();
                write!(f, "Article bytes: {len}")
            }
            ArticleState::Failed(error) => write!(f, "{error}"),
        }
    }
}

async fn request(path: &str) -> Result<String, String> {
    let response = match Request::get(path).send().await {
        Ok(response) => response,
        Err(error) => {
            return Err(format!("Connection failed: {error}"));
        }
    };

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    match response.text().await {
        Ok(text) => Ok(text),
        Err(error) => Err(format!("Failed to read response: {error}")),
    }
}

#[function_component(App)]
fn app() -> Html {
    let health = use_state(|| HealthState::Loading);
    let article = use_state(|| ArticleState::Loading);
    let article_ref = use_node_ref();

    {
        let health = health.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match request("/api/health").await {
                    Ok(resp) => health.set(HealthState::Ready(resp)),
                    Err(err) => health.set(HealthState::Failed(err)),
                }
            })
        });
    }

    {
        let article = article.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match request("/api/article/Earth").await {
                    Ok(html) => article.set(ArticleState::Ready(html)),
                    Err(err) => article.set(ArticleState::Failed(err)),
                }
            });
        });
    }

    let on_retry = {
        let health = health.clone();

        Callback::from(move |_event: MouseEvent| {
            health.set(HealthState::Loading);
            let health = health.clone();
            spawn_local(async move {
                match request("/api/health").await {
                    Ok(resp) => health.set(HealthState::Ready(resp)),
                    Err(err) => health.set(HealthState::Failed(err)),
                }
            });
        })
    };
    let is_loading = matches!(*health, HealthState::Loading);
    let article_view = match &*article {
        ArticleState::Loading => html! {
            <p>{"Article loading"}</p>
        },
        ArticleState::Ready(html) => html! {
            <iframe
                srcdoc={html.clone()}
                sandbox="allow-same-origin"
                ref={article_ref}
                title={"Wikipedia article"}
                style="width: 100%; height: 70vh;">
            </iframe>
        },
        ArticleState::Failed(err) => html! {
            <p>{format!("Error fetching article: {}", err)}</p>
        },
    };
    html! {
        <main>
            <h1>{"Wikipath"}</h1>
            <p>{ format!("Backend: {}", &*health) }</p>
            <p>{ "Find your way through Wikipedia" }</p>
            { article_view }
            <button onclick={on_retry} disabled={is_loading}>{"Retry"}</button>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
