use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    layout!(AppLayout);
    page!(Home);
}

#[component]
fn AppLayout() -> impl IntoView {
    view! { <Outlet/> }
}
#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {}
