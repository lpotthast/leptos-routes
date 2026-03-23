use leptos_routes::routes;

#[routes(without_views)]
pub mod routes {
    #[route("/orgs")]
    mod orgs {
        #[route("/:org")]
        mod org {
            #[route("/repos")]
            mod repos {
                #[route("/:repo")]
                mod repo {
                    #[route("/branches")]
                    mod branches {
                        #[route("/:branch")]
                        mod branch {}
                    }
                }
            }
        }
    }
}

fn main() {
    use assertr::prelude::*;

    // 6 levels deep: orgs -> :org -> repos -> :repo -> branches -> :branch
    assert_that!(routes::Orgs.materialize()).is_equal_to("/orgs");
    assert_that!(routes::orgs::Org.materialize("acme")).is_equal_to("/orgs/acme");
    assert_that!(routes::orgs::org::Repos.materialize("acme")).is_equal_to("/orgs/acme/repos");
    assert_that!(routes::orgs::org::repos::Repo.materialize("acme", "web"))
        .is_equal_to("/orgs/acme/repos/web");
    assert_that!(routes::orgs::org::repos::repo::Branches.materialize("acme", "web"))
        .is_equal_to("/orgs/acme/repos/web/branches");
    assert_that!(routes::orgs::org::repos::repo::branches::Branch.materialize(
        "acme", "web", "main"
    ))
    .is_equal_to("/orgs/acme/repos/web/branches/main");

    // path_pattern chains correctly.
    assert_that!(routes::orgs::org::repos::repo::branches::Branch.path_pattern())
        .is_equal_to("/orgs/:org/repos/:repo/branches/:branch");

    // All 6 routes in the enum.
    assert_eq!(routes::Route::all().len(), 6);
}
