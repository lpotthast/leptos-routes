use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    #[route("/")]
    mod root {
        layout!(|| view! { "layout 1" });
        layout!(|| view! { "layout 2" });
    }
}

fn main() {}
