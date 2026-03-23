use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(NotFound);
    layout!(MainLayout);
    index!(Home);

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
fn NotFound() -> impl IntoView {
    view! { "404" }
}

#[component]
fn MainLayout() -> impl IntoView {
    view! { <div id="main-layout"><Outlet/></div> }
}

#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

#[component]
fn UsersLayout() -> impl IntoView {
    view! { <div id="users-layout"><Outlet/></div> }
}

#[component]
fn UsersList() -> impl IntoView {
    view! { "Users List" }
}

#[component]
fn UserPage() -> impl IntoView {
    view! { "User Page" }
}

fn main() {
    // Use router() instead of manually wrapping route_tree() in <Router>.
    fn app() -> impl IntoView {
        routes::router()
    }

    let _owner = Owner::new_root(None);

    provide_context::<RequestUrl>(RequestUrl::default());
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">Home</div>"#);

    provide_context::<RequestUrl>(RequestUrl::new(
        routes::users::User.materialize("42").as_str(),
    ));
    assert_that!(app().to_html())
        .is_equal_to(r#"<div id="main-layout"><div id="users-layout">User Page</div></div>"#);
}
