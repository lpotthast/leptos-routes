use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/")]
    mod root {
        #[route("/welcome")]
        mod welcome {}

        #[route("/users/:id")]
        mod user {}
    }
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::{ParamSegment, StaticSegment};

    // path() returns the local segment tuple for each route.
    assert_that!(routes::Root.path()).is_equal_to(());
    assert_that!(routes::root::Welcome.path()).is_equal_to((StaticSegment("welcome"),));
    assert_that!(routes::root::User.path())
        .is_equal_to((StaticSegment("users"), ParamSegment("id")));

    // materialize() builds the full URL string.
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::root::Welcome.materialize()).is_equal_to("/welcome");
    assert_that!(routes::root::User.materialize("42")).is_equal_to("/users/42");

    // path_pattern() returns the complete path pattern.
    assert_that!(routes::Root.path_pattern()).is_equal_to("/");
    assert_that!(routes::root::Welcome.path_pattern()).is_equal_to("/welcome");
    assert_that!(routes::root::User.path_pattern()).is_equal_to("/users/:id");
}
