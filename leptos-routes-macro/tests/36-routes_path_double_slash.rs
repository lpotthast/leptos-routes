use leptos_routes::routes;

#[routes(without_views, path = "/api//v1")]
pub mod routes {
    #[route("/users")]
    mod users {}
}

fn main() {}
