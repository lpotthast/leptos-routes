use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

// Components named like the route structs generated for them: `Root` (for the root-level
// `layout!()`) and `Users` (for `mod users`). Glob-imported, like a component library's prelude.
mod components {
    use leptos::prelude::*;
    use leptos_router::components::Outlet;

    #[component]
    pub fn Root() -> impl IntoView {
        view! { <main><Outlet/></main> }
    }

    #[component]
    pub fn Users() -> impl IntoView {
        view! { "Users" }
    }
}
use components::*;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    // View expressions resolve names in the enclosing scope first: These are the components.
    layout!(Root);
    index!(|| view! { <a href=routes::Users.materialize()>"Users"</a> });

    #[route("/users")]
    mod users {
        page!(Users);
    }

    // Named like `leptos::prelude::Action`.
    #[route("/action")]
    mod action {
        page!(|| view! { "Action" });
    }
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

    provide_context::<RequestUrl>(RequestUrl::default());
    assert_that!(app().to_html()).is_equal_to(r#"<main><a href="/users">Users</a></main>"#);

    provide_context::<RequestUrl>(RequestUrl::new(routes::Users.materialize().as_str()));
    assert_that!(app().to_html()).is_equal_to("<main>Users</main>");

    provide_context::<RequestUrl>(RequestUrl::new(routes::Action.materialize().as_str()));
    assert_that!(app().to_html()).is_equal_to("<main>Action</main>");
}
