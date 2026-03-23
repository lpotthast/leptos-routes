use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/")]
    mod root {
        #[route("/users")]
        mod users {
            #[route("/:id")]
            mod user {}
        }
    }
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::ParamSegment;

    // All types are accessible despite modules being declared without `pub`.
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::root::Users.materialize()).is_equal_to("/users");
    assert_that!(routes::root::users::User.path()).is_equal_to((ParamSegment("id"),));
    assert_that!(routes::root::users::User.materialize("42")).is_equal_to("/users/42");

    let _route: routes::Route = routes::Route::RootUsersUser(routes::root::users::User);
}
