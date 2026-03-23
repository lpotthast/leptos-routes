use leptos_routes::routes;

// Dummy values for layout!/index! expressions.
fn root_layout() {}
fn root_index() {}

#[routes(without_views)]
pub mod routes {
    layout!(root_layout);
    index!(root_index);

    #[route("/*rest")]
    mod catch_all {}
}

fn main() {
    use assertr::prelude::*;

    // Synthetic root at "/".
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/");

    // Wildcard child.
    assert_that!(routes::CatchAll.materialize("any/path/here"))
        .is_equal_to("/any/path/here");
    assert_that!(routes::CatchAll.path_pattern()).is_equal_to("/*rest");

    // Enum has Root + CatchAll.
    assert_eq!(routes::Route::all().len(), 2);

    let _: routes::Route = routes::Route::Root(routes::Root);
    let _: routes::Route = routes::Route::CatchAll(routes::CatchAll);
}
