use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Router};
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });

    #[route("/")]
    mod root {
        layout!(MainLayout);
        index!(Dashboard);

        // This parent has NO layout!() — should get an implicit <Outlet/> passthrough.
        #[route("/users")]
        mod users {
            index!(UsersList);

            #[route("/:id")]
            mod user {
                page!(UserPage);
            }
        }
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    view! { <div id="main-layout"><Outlet/></div> }
}
#[component]
fn Dashboard() -> impl IntoView {
    view! { "Dashboard" }
}
#[component]
fn UsersList() -> impl IntoView {
    view! { "UsersList" }
}
#[component]
fn UserPage() -> impl IntoView {
    view! { "UserPage" }
}

fn main() {
    fn app() -> impl IntoView {
        view! {
            <Router>
                { routes::route_tree() }
            </Router>
        }
    }

    let _owner = Owner::new_root(None);

    // When visiting /users, the implicit Outlet should pass through directly
    // (no wrapping div from users, just main-layout wrapping).
    provide_context::<RequestUrl>(RequestUrl::new("/users"));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">UsersList</div>"#);

    // When visiting /users/42, the implicit Outlet on /users should pass through.
    provide_context::<RequestUrl>(RequestUrl::new("/users/42"));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">UserPage</div>"#);
}
