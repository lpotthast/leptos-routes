use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    // All four segment types in one route, including the Rust keyword `type`.
    #[route("/complex/:foo/:type?/*baz")]
    mod complex {}

    // Simple static-only route for contrast.
    #[route("/foo/bar")]
    mod static_multi {}
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::{OptionalParamSegment, ParamSegment, StaticSegment, WildcardSegment};

    // Static segments.
    assert_that!(routes::StaticMulti.path())
        .is_equal_to((StaticSegment("foo"), StaticSegment("bar")));
    assert_that!(routes::StaticMulti.materialize()).is_equal_to("/foo/bar");
    assert_that!(routes::StaticMulti.path_pattern()).is_equal_to("/foo/bar");

    // All segment types: static, param, optional param, wildcard.
    assert_that!(routes::Complex.path()).is_equal_to((
        StaticSegment("complex"),
        ParamSegment("foo"),
        OptionalParamSegment("type"),
        WildcardSegment("baz"),
    ));

    // Keyword escaping: `:type` becomes the `type_` parameter in materialize().
    // Optional param with Some value.
    assert_that!(routes::Complex.materialize("42", Some("ok"), "bob"))
        .is_equal_to("/complex/42/ok/bob");

    // Optional param with None.
    assert_that!(routes::Complex.materialize("42", None::<&str>, "otto"))
        .is_equal_to("/complex/42/otto");

    assert_that!(routes::Complex.path_pattern()).is_equal_to("/complex/:foo/:type?/*baz");
}
