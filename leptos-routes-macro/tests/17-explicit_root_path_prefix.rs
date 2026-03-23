use leptos_routes::routes;

#[routes(without_views, path = "/")]
pub mod routes {
    #[route("/users")]
    mod users {}
}

fn main() {
    use assertr::prelude::*;

    // path = "/" creates a synthetic Root.
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/");

    // Children work as normal.
    assert_that!(routes::Users.materialize()).is_equal_to("/users");
    assert_that!(routes::Users.path_pattern()).is_equal_to("/users");
}
