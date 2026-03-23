use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/")]
    mod root {

        #[route("/users")]
        mod users {

            #[route("/:id")]
            mod user {

                #[route("/addresses")]
                mod addresses {

                    #[route("/:id")]
                    mod address {}
                }
            }
        }
    }
}

fn main() {}
