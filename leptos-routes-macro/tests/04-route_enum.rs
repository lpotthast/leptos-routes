use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/")]
    mod root {
        #[route("/about")]
        mod about {}

        #[route("/users")]
        mod users {
            #[route("/:id")]
            mod user {}
        }
    }
}

fn main() {
    use assertr::prelude::*;

    // Exhaustive match — all variants exist.
    let route: routes::Route = routes::Route::RootUsersUser(routes::root::users::User);
    match route {
        routes::Route::Root(_) => {}
        routes::Route::RootAbout(_) => {}
        routes::Route::RootUsers(_) => {}
        routes::Route::RootUsersUser(_) => {}
    }

    // path_pattern() returns the full path pattern.
    assert_that!(route.path_pattern()).is_equal_to("/users/:id");

    // Display impl matches path_pattern().
    assert_that!(format!("{}", route).as_str()).is_equal_to("/users/:id");

    // all() returns every variant.
    let all_routes = routes::Route::all();
    assert_eq!(all_routes.len(), 4);

    // Debug, Clone, Copy are derived.
    let _debug = format!("{:?}", route);
    let route_copy = route;
    let route_clone = route.clone();
    assert_that!(route_copy).is_equal_to(route);
    assert_that!(route_clone).is_equal_to(route);

    // PartialEq: same route equals itself.
    assert_that!(routes::Route::Root(routes::Root)).is_equal_to(routes::Route::Root(routes::Root));

    // Hash is derived — routes can be used as HashMap/HashSet keys.
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(routes::Route::Root(routes::Root));
    set.insert(routes::Route::Root(routes::Root)); // duplicate
    assert_that!(set.len()).is_equal_to(1);

    // Individual structs are also hashable.
    let mut struct_set = HashSet::new();
    struct_set.insert(routes::Root);
    assert_that!(struct_set.len()).is_equal_to(1);
}
