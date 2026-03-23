use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/*rest/foo")]
    mod bad_wildcard {}
}

fn main() {}
