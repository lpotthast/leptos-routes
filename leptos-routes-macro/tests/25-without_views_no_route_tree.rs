use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {

    #[route("/")]
    mod root {}
}

fn main() {
    // When `without_views` is set, `route_tree()` is not generated.
    // Attempting to call it should produce a compile error.
    let _ = routes::route_tree();
}
