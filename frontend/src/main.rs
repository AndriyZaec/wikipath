use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
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
