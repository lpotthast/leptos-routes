use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    #[route("/")]
    mod root {
        layout!(Root);
        page!(Home);
    }
}

#[component]
fn Root() -> impl IntoView {
    view! { "Root" }
}
#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {}
