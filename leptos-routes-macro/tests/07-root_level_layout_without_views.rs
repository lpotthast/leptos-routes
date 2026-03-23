use leptos_routes::routes;

// Dummy values for layout!/index! expressions (they expand to hidden functions
// that reference the expression, even with without_views).
fn main_layout() {}
fn home() {}

#[routes(without_views)]
pub mod routes {
    layout!(main_layout);
    index!(home);

    #[route("/users")]
    mod users {
        #[route("/:id")]
        mod user {}
    }

    #[route("/about")]
    mod about {}
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::StaticSegment;

    // Synthetic Root struct is generated.
    assert_that!(routes::Root.path()).is_equal_to(());
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/");

    // Children are directly under routes::, not routes::root::
    assert_that!(routes::Users.path()).is_equal_to((StaticSegment("users"),));
    assert_that!(routes::Users.materialize()).is_equal_to("/users");
    assert_that!(routes::Users.path_pattern()).is_equal_to("/users");

    assert_that!(routes::users::User.materialize("42")).is_equal_to("/users/42");
    assert_that!(routes::users::User.path_pattern()).is_equal_to("/users/:id");

    assert_that!(routes::About.path()).is_equal_to((StaticSegment("about"),));
    assert_that!(routes::About.materialize()).is_equal_to("/about");

    // Enum has all 4 variants.
    assert_eq!(routes::Route::all().len(), 4);

    // Enum variant names don't include "Root" prefix for children.
    let _: routes::Route = routes::Route::Root(routes::Root);
    let _: routes::Route = routes::Route::Users(routes::Users);
    let _: routes::Route = routes::Route::UsersUser(routes::users::User);
    let _: routes::Route = routes::Route::About(routes::About);
}
