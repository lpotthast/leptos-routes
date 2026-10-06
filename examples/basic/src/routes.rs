//! The routes of the app, referred to as `routes::Users`, `routes::users::User`, ...
//!
//! `#[routes]` can't annotate this file module itself. It annotates the private module `defs`
//! instead, whose generated items are re-exported into this one. Import nothing else into this
//! scope: Every name here competes with the re-exported route structs.

use leptos_routes::routes;

#[routes]
mod defs {
    use crate::pages;

    fallback!(pages::NotFound);
    layout!(pages::MainLayout);
    index!(pages::Home);

    #[route("/users")]
    mod users {
        layout!(pages::UsersLayout);
        index!(pages::UsersList);

        #[route("/:id")]
        mod user {
            page!(pages::UserPage);
        }
    }
}
pub use defs::*;
