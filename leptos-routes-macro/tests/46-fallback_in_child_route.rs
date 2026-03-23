use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });

    #[route("/")]
    mod root {
        fallback!(|| view! { "nope" });
    }
}

fn main() {}
