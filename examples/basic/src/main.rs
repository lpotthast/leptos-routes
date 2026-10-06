use leptos::prelude::*;

mod pages;
mod routes;

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
