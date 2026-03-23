use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    page!(Home);

    #[route("/users")]
    mod users {}
}

fn main() {}
