use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

enum HealthState {
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
    html! {
        <main>
            <h1>{"Wikipath"}</h1>
            <p>{ format!("Backend: {}", &*health) }</p>
            <p>{ "Find your way through Wikipedia" }</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
