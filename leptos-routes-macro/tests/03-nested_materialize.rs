use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/categories")]
    mod categories {
        #[route("/:category")]
        mod category {
            #[route("/:item_id")]
            mod item {}
        }
    }
}

fn main() {
    use assertr::prelude::*;
    use leptos_router::{ParamSegment, StaticSegment};

    // Parent route without params.
    assert_that!(routes::Categories.path()).is_equal_to((StaticSegment("categories"),));
    assert_that!(routes::Categories.materialize()).is_equal_to("/categories");

    // Child with one param inherits parent's path.
    assert_that!(routes::categories::Category.path()).is_equal_to((ParamSegment("category"),));
    assert_that!(routes::categories::Category.materialize("books"))
        .is_equal_to("/categories/books");

    // Grandchild: materialize() chains params through the full hierarchy.
    assert_that!(routes::categories::category::Item.path()).is_equal_to((ParamSegment("item_id"),));
    assert_that!(routes::categories::category::Item.materialize("books", "123"))
        .is_equal_to("/categories/books/123");

    // path_pattern() includes all ancestor segments.
    assert_that!(routes::Categories.path_pattern()).is_equal_to("/categories");
    assert_that!(routes::categories::Category.path_pattern()).is_equal_to("/categories/:category");
    assert_that!(routes::categories::category::Item.path_pattern())
        .is_equal_to("/categories/:category/:item_id");

    // materialize() accepts any Display type, not just &str.
    assert_that!(routes::categories::Category.materialize(42u32))
        .is_equal_to("/categories/42");
    assert_that!(routes::categories::category::Item.materialize(1u32, 2u32))
        .is_equal_to("/categories/1/2");
}
