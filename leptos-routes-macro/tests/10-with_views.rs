use assertr::prelude::*;
use leptos::prelude::*;
use leptos_router::components::{Outlet, Router};
use leptos_router::location::RequestUrl;
use leptos_routes::routes;

#[routes]
pub mod routes {
    fallback!(|| view! { <FallbackComponent/> });

    // A route without any segment.
    #[route("/")]
    mod root {
        layout!(MainLayout);
        index!(PageDashboard);

        // A route with a single static segment.
        #[route("/welcome")]
        mod welcome {
            page!(PageWelcome);
        }

        // A route with multiple static segments.
        #[route("/foo/bar")]
        mod multiple_static {
            page!(SomePage);
        }

        // A route with multiple segments, not being all static.
        #[route("/foo/:bar")]
        mod multiple_dynamic {
            page!(SomePage);
        }

        // A route with all types of segments.
        // This route also uses the rust keyword `type` that must be handled.
        #[route("/complex/:foo/:type?/*baz")]
        mod complex {
            page!(SomePage);
        }

        // Nested routes.
        #[route("/users")]
        mod users {
            layout!(UsersLayout);
            index!(NoUser);

            // The 'layout' attribute on modules containing children is optional!
            #[route("/:id")]
            mod user {
                layout!(UserLayout);
                index!(User);

                // This has the same name as a root-level route. That must not lead to a name clash!
                #[route("/settings")]
                mod welcome {
                    page!(UserSettings);
                }

                #[route("/details")]
                mod details {
                    page!(UserDetails);
                }
            }
        }
    }
}

#[component]
fn FallbackComponent() -> impl IntoView {
    view! {
        "Fallback"
    }
}

#[component]
fn MainLayout() -> impl IntoView {
    view! {
        <div id="main-layout">
            <Outlet/>
        </div>
    }
}

#[component]
fn UsersLayout() -> impl IntoView {
    view! {
        <div id="users-layout">
            <Outlet/>
        </div>
    }
}

#[component]
fn UserLayout() -> impl IntoView {
    view! {
        <div id="user-layout">
            <Outlet/>
        </div>
    }
}

#[component]
fn PageDashboard() -> impl IntoView {
    view! {
        "Dashboard"
    }
}

#[component]
fn PageWelcome() -> impl IntoView {
    view! {
        "Welcome"
    }
}

#[component]
fn NoUser() -> impl IntoView {
    view! {
        "NoUser"
    }
}

#[component]
fn User() -> impl IntoView {
    view! {
        "User"
    }
}

#[component]
fn UserSettings() -> impl IntoView {
    view! {
        "UserSettings"
    }
}

#[component]
fn UserDetails() -> impl IntoView {
    view! {
        "UserDetails"
    }
}

#[component]
fn SomePage() -> impl IntoView {
    view! {
        "SomePage"
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
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">Dashboard</div>"#);

    provide_context::<RequestUrl>(RequestUrl::new(
        routes::root::Welcome.materialize().as_str(),
    ));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout">Welcome</div>"#);

    provide_context::<RequestUrl>(RequestUrl::new(
        routes::root::users::user::Details
            .materialize("42")
            .as_str(),
    ));
    assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout"><div id="users-layout"><div id="user-layout">UserDetails</div></div></div>"#);
}
