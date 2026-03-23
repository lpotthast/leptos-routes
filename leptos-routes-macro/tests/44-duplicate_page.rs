use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });

    #[route("/")]
    mod root {
        page!(Home);
        page!(AnotherHome);
    }
}

#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

#[component]
fn AnotherHome() -> impl IntoView {
    view! { "Other" }
}

fn main() {}
