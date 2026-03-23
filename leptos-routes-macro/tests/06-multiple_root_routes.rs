use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/api")]
    mod api {
        #[route("/users")]
        mod users {}
    }

    #[route("/app")]
    mod app {
        #[route("/dashboard")]
        mod dashboard {}
    }
}

fn main() {
    use assertr::prelude::*;

    // Two independent root routes
    assert_that!(routes::Api.materialize()).is_equal_to("/api");
    assert_that!(routes::App.materialize()).is_equal_to("/app");

    // Children resolve correctly under each root
    assert_that!(routes::api::Users.materialize()).is_equal_to("/api/users");
    assert_that!(routes::app::Dashboard.materialize()).is_equal_to("/app/dashboard");

    // path_pattern works correctly
    assert_that!(routes::Api.path_pattern()).is_equal_to("/api");
    assert_that!(routes::api::Users.path_pattern()).is_equal_to("/api/users");
    assert_that!(routes::App.path_pattern()).is_equal_to("/app");
    assert_that!(routes::app::Dashboard.path_pattern()).is_equal_to("/app/dashboard");

    // Enum has all 4 variants
    assert_eq!(routes::Route::all().len(), 4);

    // Variant names don't collide
    let _: routes::Route = routes::Route::Api(routes::Api);
    let _: routes::Route = routes::Route::ApiUsers(routes::api::Users);
    let _: routes::Route = routes::Route::App(routes::App);
    let _: routes::Route = routes::Route::AppDashboard(routes::app::Dashboard);
}
