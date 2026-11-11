use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let health = use_state(|| String::from("Checking backennd..."));

    {
        let health = health.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let response = match Request::get("/api/health").send().await {
                    Ok(response) => response,
                    Err(error) => {
                        health.set(format!("Connection failed: {error}"));
                        return;
                    }
                };

                if !response.ok() {
                    health.set(format!("HTTP error: {}", response.status()));
                    return;
                }

                let text = match response.text().await {
                    Ok(text) => text,
                    Err(error) => {
                        health.set(format!("Failed to read response: {error}"));
                        return;
                    }
                };

                health.set(text);
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
