use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { "404" });
    page!(Home);
}

#[component]
fn Home() -> impl IntoView {
    view! { "Home" }
}

fn main() {
    use assertr::prelude::*;

    // Root as leaf route.
    assert_that!(routes::Root.materialize()).is_equal_to("/");
    assert_that!(routes::Root.path_pattern()).is_equal_to("/");

    assert_eq!(routes::Route::all().len(), 1);

    let _: routes::Route = routes::Route::Root(routes::Root);

    fn app() -> impl IntoView {
        view! {
            <Router>
                { routes::route_tree() }
            </Router>
        }
    }

    let _owner = Owner::new_root(None);

    provide_context::<RequestUrl>(RequestUrl::default());
    assert_that!(app().to_html()).is_equal_to("Home");
}
