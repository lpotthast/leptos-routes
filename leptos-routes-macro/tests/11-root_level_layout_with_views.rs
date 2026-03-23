use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Router};
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    layout!(MainLayout);
    index!(Dashboard);

    #[route("/users")]
    mod users {
        layout!(UsersLayout);
        index!(UsersList);

        #[route("/:id")]
        mod user {
            page!(UserPage);
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
fn UsersLayout() -> impl IntoView {
    view! { <div id="users-layout"><Outlet/></div> }
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

    // Root index renders at "/"
    provide_context::<RequestUrl>(RequestUrl::default());
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">Dashboard</div>"#);

    // Nested route renders with layouts
    provide_context::<RequestUrl>(RequestUrl::new(
        routes::users::User.materialize("42").as_str(),
    ));
    assert_that!(app().to_html()).is_equal_to(
        r#"<div id="main-layout"><div id="users-layout">UserPage</div></div>"#,
    );
}
