use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/users")]
    mod users {
        index!(UsersList);

        #[route("/")]
        mod index {}

        #[route("/:id")]
        mod user {}
    }
}

fn main() {}
