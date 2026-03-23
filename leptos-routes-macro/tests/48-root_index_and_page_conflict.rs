use leptos::prelude::*;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    index!(Dashboard);
    page!(Home);
}

#[component]
fn Dashboard() -> impl IntoView {
    view! { "Dashboard" }
}
#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {}
