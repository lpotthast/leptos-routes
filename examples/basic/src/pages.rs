use leptos::prelude::*;
use leptos_router::components::Outlet;

use crate::routes;

#[component]
pub fn NotFound() -> impl IntoView {
    view! { <h1>"404 - Not Found"</h1> }
}

#[component]
pub fn MainLayout() -> impl IntoView {
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
pub fn Home() -> impl IntoView {
    view! { <h1>"Home"</h1><p>"Welcome to the leptos-routes basic example."</p> }
}

#[component]
pub fn UsersLayout() -> impl IntoView {
    view! {
        <div id="users">
            <Outlet/>
        </div>
    }
}

#[component]
pub fn UsersList() -> impl IntoView {
    view! {
        <h1>"Users"</h1>
        <ul>
            <li><a href=routes::users::User.materialize(42)>"User 42"</a></li>
        </ul>
    }
}

#[component]
pub fn UserPage() -> impl IntoView {
    view! { <h1>"User Page"</h1> }
}
