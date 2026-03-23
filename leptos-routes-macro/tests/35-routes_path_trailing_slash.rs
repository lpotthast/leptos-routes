use leptos_routes::routes;

#[routes(without_views, path = "/api/")]
pub mod routes {
    #[route("/users")]
    mod users {}
}

fn main() {}
