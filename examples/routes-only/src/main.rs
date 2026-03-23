use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    /// Static root route.
    #[route("/")]
    mod root {}

    /// Parent route with a static segment.
    #[route("/users")]
    mod users {
        /// Dynamic parameter route.
        #[route("/:id")]
        mod user {
            /// Leaf route with a static segment.
            #[route("/details")]
            mod details {}
        }
    }

    /// Route with an optional parameter.
    #[route("/search/:query?")]
    mod search {}

    /// Route with a wildcard segment.
    #[route("/files/*path")]
    mod files {}
}

fn main() {
    // === Type-safe URL construction ===
    //
    // Each route struct has a `materialize()` method that builds a full URL path.
    // Parameters accept any type that implements `Display`.
    println!("--- Type-safe URL construction ---\n");

    // No parameters needed for static routes.
    println!("  Root URL:    {}", routes::Root.materialize());
    println!("  Users URL:   {}", routes::Users.materialize());

    // Dynamic segments accept any Display type.
    println!("  User 42:     {}", routes::users::User.materialize(42u32));
    println!("  User \"abc\":  {}", routes::users::User.materialize("abc"));

    // Parameters accumulate through the hierarchy — child routes include parent params.
    println!(
        "  Details:     {}",
        routes::users::user::Details.materialize(7)
    );

    // Optional parameters use Option — None omits the segment.
    println!(
        "  Search:      {}",
        routes::Search.materialize(Some("rust"))
    );
    println!(
        "  Search none: {}",
        routes::Search.materialize(None::<&str>)
    );

    // Wildcard segments capture the rest of the path.
    println!(
        "  Files:       {}",
        routes::Files.materialize("docs/readme.md")
    );

    // === Path patterns ===
    //
    // `path()` returns the local segment(s) for use in <Route> declarations.
    // `path_pattern()` returns the full path including all parent segments.
    println!("\n--- Path patterns ---\n");

    println!("  {:20} {:30}", "Route", "Full pattern");
    println!("  {:20} {:30}", "-----", "------------");
    println!(
        "  {:20} {:30}",
        "Root",
        routes::Root.path_pattern()
    );
    println!(
        "  {:20} {:30}",
        "Users",
        routes::Users.path_pattern()
    );
    println!(
        "  {:20} {:30}",
        "users::User",
        routes::users::User.path_pattern()
    );
    println!(
        "  {:20} {:30}",
        "users::user::Details",
        routes::users::user::Details.path_pattern()
    );
    println!(
        "  {:20} {:30}",
        "Search",
        routes::Search.path_pattern()
    );
    println!(
        "  {:20} {:30}",
        "Files",
        routes::Files.path_pattern()
    );

    // === Route enum ===
    //
    // The generated `Route` enum has a variant for every route.
    // Use it for exhaustive matching, logging, or building route tables.
    println!("\n--- Route enum ---\n");

    println!("  All {} routes:", routes::Route::all().len());
    for route in routes::Route::all() {
        // Display delegates to path_pattern().
        println!("    {route}");
    }

    // Pattern matching on the enum.
    let route = routes::Route::UsersUserDetails(routes::users::user::Details);
    match route {
        routes::Route::Root(_) => println!("\n  Matched: root"),
        routes::Route::UsersUserDetails(r) => {
            println!("\n  Matched: user details (pattern: {})", r.path_pattern())
        }
        _ => println!("\n  Matched: other route"),
    }
}
