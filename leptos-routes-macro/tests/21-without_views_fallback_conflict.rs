use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    fallback!(|| "not found");

    #[route("/")]
    mod root {}
}

fn main() {}
