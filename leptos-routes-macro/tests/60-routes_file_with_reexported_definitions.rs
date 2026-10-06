//! The recommended layout: route definitions in their own `routes.rs` file, in a private module
//! whose items are re-exported, so that routes are referred to as `routes::Users`.

use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;

mod pages {
    use leptos::prelude::*;
    use leptos_router::components::Outlet;

    #[component]
    pub fn Err404() -> impl IntoView {
        view! { "404" }
    }

    #[component]
    pub fn MainLayout() -> impl IntoView {
        view! { <main><Outlet/></main> }
    }

    #[component]
    pub fn Dashboard() -> impl IntoView {
        view! { <a href=crate::routes::users::User.materialize(7)>"User 7"</a> }
    }

    // Named like the route struct of `mod users`.
    #[component]
    pub fn Users() -> impl IntoView {
        view! { "Users" }
    }

    // Named like the route struct of `mod user`.
    #[component]
    pub fn User() -> impl IntoView {
        view! { "User" }
    }
}

// Stands in for a `src/routes.rs` file, declared with `mod routes;`. Nothing but the macro is
// imported here: Every name in this scope competes with the re-exported route structs.
mod routes {
    use leptos_routes::routes;

    #[routes]
    mod defs {
        use crate::pages;
        use leptos::prelude::*;

        fallback!(|| view! { <pages::Err404/> });
        layout!(pages::MainLayout);
        index!(pages::Dashboard);

        #[route("/users")]
        mod users {
            index!(pages::Users);

            #[route("/:id")]
            mod user {
                page!(pages::User);
            }
        }
    }
    pub use defs::*;
}

fn main() {
    fn app() -> impl IntoView {
        view! {
            <Router>
                { routes::route_tree() }
            </Router>
        }
    }

    assert_that!(routes::Users.materialize()).is_equal_to("/users");
    assert_that!(routes::users::User.materialize(7)).is_equal_to("/users/7");
    assert_that!(routes::Route::all()).has_length(3);

    let _owner = Owner::new_root(None);

    provide_context::<RequestUrl>(RequestUrl::default());
    assert_that!(app().to_html()).is_equal_to(r#"<main><a href="/users/7">User 7</a></main>"#);

    provide_context::<RequestUrl>(RequestUrl::new(routes::Users.materialize().as_str()));
    assert_that!(app().to_html()).is_equal_to("<main>Users</main>");

    provide_context::<RequestUrl>(RequestUrl::new(
        routes::users::User.materialize(7).as_str(),
    ));
    assert_that!(app().to_html()).is_equal_to("<main>User</main>");
}
