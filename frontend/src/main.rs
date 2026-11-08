use core::assert_eq;

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    use_effect_with((), |_| {
        spawn_local(async {
            let response = Request::get("/api/health")
                .send()
                .await
                .expect("Health request failed");

            let text = response
                .text()
                .await
                .expect("Failed to read health response");

            assert_eq!(text, "ok");
        })
    });
    html! {
        <main>
            <h1>{"Wikipath"}</h1>
            <p>{ "Find your way through Wikipedia" }</p>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
