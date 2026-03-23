# TODO

- [ ] A `#[route("/foo")` annotated `mod foo` redundantly specifies "foo". We could allow users to omit a path string
  entirely, assuming a static path segment with the name of the module is sufficient in that case.
