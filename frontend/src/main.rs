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

                let text = response
                    .text()
                    .await
                    .expect("Failed to read health response");

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
