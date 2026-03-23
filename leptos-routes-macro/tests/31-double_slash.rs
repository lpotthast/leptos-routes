use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/users//posts")]
    mod users_posts {}
}

fn main() {}
