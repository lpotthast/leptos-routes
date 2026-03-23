use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("users")]
    mod users {}
}

fn main() {}
