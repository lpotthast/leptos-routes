use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Router};
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes(path = "/api")]
pub mod routes {
    fallback!(|| view! { "404" });
    layout!(ApiLayout);
    index!(ApiIndex);

    #[route("/users")]
    mod users {
        page!(UsersPage);
    }
}

#[component]
fn ApiLayout() -> impl IntoView {
    view! { <div id="api-layout"><Outlet/></div> }
}

#[component]
fn ApiIndex() -> impl IntoView {
    view! { "ApiIndex" }
}

#[component]
fn UsersPage() -> impl IntoView {
    view! { "UsersPage" }
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

    // /api renders the index within the layout.
    provide_context::<RequestUrl>(RequestUrl::new("/api"));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="api-layout">ApiIndex</div>"#);

    // /api/users renders the users page within the layout.
    provide_context::<RequestUrl>(RequestUrl::new(
        routes::Users.materialize().as_str(),
    ));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="api-layout">UsersPage</div>"#);
}
