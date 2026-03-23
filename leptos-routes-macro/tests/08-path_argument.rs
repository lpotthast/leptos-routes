use leptos_routes::routes;

#[routes(without_views, path = "/api")]
pub mod routes {
    #[route("/users")]
    mod users {
        #[route("/:id")]
        mod user {}
    }

    #[route("/health")]
    mod health {}
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::StaticSegment;

    // Synthetic Root represents the path prefix.
    assert_that!(routes::Root.path()).is_equal_to((StaticSegment("api"),));
    assert_that!(routes::Root.materialize()).is_equal_to("/api");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/api");

    // Children include the prefix in materialize.
    assert_that!(routes::Users.materialize()).is_equal_to("/api/users");
    assert_that!(routes::Users.path_pattern()).is_equal_to("/api/users");

    assert_that!(routes::users::User.materialize("42")).is_equal_to("/api/users/42");
    assert_that!(routes::users::User.path_pattern()).is_equal_to("/api/users/:id");

    assert_that!(routes::Health.materialize()).is_equal_to("/api/health");

    // Enum has Root + all children
    assert_eq!(routes::Route::all().len(), 4);

    let _: routes::Route = routes::Route::Root(routes::Root);
    let _: routes::Route = routes::Route::Users(routes::Users);
    let _: routes::Route = routes::Route::UsersUser(routes::users::User);
    let _: routes::Route = routes::Route::Health(routes::Health);
}
