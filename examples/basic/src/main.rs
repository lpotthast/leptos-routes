use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_routes::routes;

#[component]
fn NotFound() -> impl IntoView {
    view! { <h1>"404 - Not Found"</h1> }
}

#[component]
fn MainLayout() -> impl IntoView {
    view! {
        <nav>
            <a href=routes::Root.materialize()>"Home"</a>
            " | "
            <a href=routes::Users.materialize()>"Users"</a>
        </nav>
        <main>
            <Outlet/>
        </main>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! { <h1>"Home"</h1><p>"Welcome to the leptos-routes basic example."</p> }
}

#[component]
fn UsersLayout() -> impl IntoView {
    view! {
        <div id="users">
            <Outlet/>
        </div>
    }
}

#[component]
fn UsersList() -> impl IntoView {
    view! {
        <h1>"Users"</h1>
        <ul>
            <li><a href=routes::users::User.materialize(42)>"User 42"</a></li>
        </ul>
    }
}

#[component]
fn UserPage() -> impl IntoView {
    view! { <h1>"User Page"</h1> }
}

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
fn App() -> impl IntoView {
    routes::router()
}

fn shell(_options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <title>"leptos-routes basic example"</title>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[tokio::main]
async fn main() {
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let leptos_options = LeptosOptions::builder()
        .output_name("basic")
        .site_root("./target/site")
        .site_pkg_dir("pkg")
        .build();

    let routes = generate_route_list(App);

    let app = axum::Router::new()
        .leptos_routes(&leptos_options, routes, {
            let options = leptos_options.clone();
            move || shell(options.clone())
        })
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on http://127.0.0.1:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
