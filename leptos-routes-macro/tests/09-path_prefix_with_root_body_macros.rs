use leptos_routes::routes;

// Dummy values for layout!/index! expressions.
fn api_layout() {}
fn api_index() {}

#[routes(without_views, path = "/api")]
pub mod routes {
    layout!(api_layout);
    index!(api_index);

    #[route("/users")]
    mod users {
        #[route("/:id")]
        mod user {}
    }
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::StaticSegment;

    // Synthetic root at /api has the layout/index.
    assert_that!(routes::Root.path()).is_equal_to((StaticSegment("api"),));
    assert_that!(routes::Root.materialize()).is_equal_to("/api");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/api");

    // Children nested under /api.
    assert_that!(routes::Users.materialize()).is_equal_to("/api/users");
    assert_that!(routes::Users.path_pattern()).is_equal_to("/api/users");

    assert_that!(routes::users::User.materialize("42")).is_equal_to("/api/users/42");
    assert_that!(routes::users::User.path_pattern()).is_equal_to("/api/users/:id");

    // Enum has Root + Users + User.
    assert_eq!(routes::Route::all().len(), 3);

    let _: routes::Route = routes::Route::Root(routes::Root);
    let _: routes::Route = routes::Route::Users(routes::Users);
    let _: routes::Route = routes::Route::UsersUser(routes::users::User);
}
