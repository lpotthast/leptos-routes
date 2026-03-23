use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    layout!(MainLayout);

    #[route("/")]
    mod root {
        #[route("/users")]
        mod users {}
    }
}

fn main() {}
