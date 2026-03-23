use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });

    #[route("/")]
    mod root {
        index!(Home);
        page!(Home);
    }
}

#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {}
