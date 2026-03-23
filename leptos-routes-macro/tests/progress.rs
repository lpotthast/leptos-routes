#[test]
fn tests() {
    let t = trybuild::TestCases::new();

    // Group 1: Core struct generation
    t.pass("tests/01-basic_route_struct.rs");
    t.pass("tests/02-segment_types.rs");
    t.pass("tests/03-nested_materialize.rs");
    t.pass("tests/04-route_enum.rs");
    t.pass("tests/05-non_pub_modules.rs");
    t.pass("tests/06-multiple_root_routes.rs");
    t.pass("tests/07-root_level_layout_without_views.rs");
    t.pass("tests/08-path_argument.rs");
    t.pass("tests/09-path_prefix_with_root_body_macros.rs");

    // Group 2: View rendering and extended struct tests
    t.pass("tests/10-with_views.rs");
    t.pass("tests/11-root_level_layout_with_views.rs");
    t.pass("tests/12-root_page_leaf.rs");
    t.pass("tests/13-implicit_outlet_passthrough.rs");
    t.pass("tests/14-deep_nesting.rs");
    t.pass("tests/15-wildcard_on_synthetic_root.rs");
    t.pass("tests/16-path_prefix_with_views.rs");
    t.pass("tests/17-explicit_root_path_prefix.rs");
    t.pass("tests/18-router_fn.rs");

    // Group 3: #[routes()] attribute errors
    t.compile_fail("tests/20-bare_routes_error.rs");
    t.compile_fail("tests/21-without_views_fallback_conflict.rs");
    t.compile_fail("tests/22-with_views_migration_error.rs");
    t.compile_fail("tests/23-fallback_migration_error.rs");
    t.compile_fail("tests/24-unknown_routes_attribute.rs");
    t.compile_fail("tests/25-without_views_no_route_tree.rs");
    t.compile_fail("tests/26-module_without_body.rs");

    // Group 4: #[route()] path validation errors
    t.compile_fail("tests/30-trailing_slash.rs");
    t.compile_fail("tests/31-double_slash.rs");
    t.compile_fail("tests/32-missing_leading_slash.rs");
    t.compile_fail("tests/33-wildcard_not_last_segment.rs");
    t.compile_fail("tests/34-param_name_collision.rs");
    t.compile_fail("tests/35-routes_path_trailing_slash.rs");
    t.compile_fail("tests/36-routes_path_double_slash.rs");

    // Group 5: Body macro conflicts & duplicates
    t.compile_fail("tests/40-layout_and_page_conflict.rs");
    t.compile_fail("tests/41-index_and_page_conflict.rs");
    t.compile_fail("tests/42-duplicate_layout.rs");
    t.compile_fail("tests/43-duplicate_index.rs");
    t.compile_fail("tests/44-duplicate_page.rs");
    t.compile_fail("tests/45-duplicate_fallback.rs");
    t.compile_fail("tests/46-fallback_in_child_route.rs");
    t.compile_fail("tests/47-root_layout_and_page_conflict.rs");
    t.compile_fail("tests/48-root_index_and_page_conflict.rs");

    // Group 6: Structural/hierarchy errors
    t.compile_fail("tests/50-index_and_index_route_conflict.rs");
    t.compile_fail("tests/51-page_on_parent_route.rs");
    t.compile_fail("tests/52-pub_route_module_error.rs");
    t.compile_fail("tests/53-leaf_route_without_page.rs");
    t.compile_fail("tests/54-root_macros_with_route_slash_conflict.rs");
    t.compile_fail("tests/55-root_page_with_children_error.rs");
}
