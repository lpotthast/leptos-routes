use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });

    #[route("/")]
    mod root {
        page!(Home);
    }

    #[route("/about")]
    mod about {}
}

#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {}
