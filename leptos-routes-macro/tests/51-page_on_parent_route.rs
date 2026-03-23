use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/users")]
    mod users {
        page!(UsersPage);

        #[route("/:id")]
        mod user {}
    }
}

fn main() {}
