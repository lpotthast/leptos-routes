use leptos_routes::routes;

#[routes(fallback = || "not found")]
pub mod routes {
    #[route("/")]
    mod root {}
}

fn main() {}
