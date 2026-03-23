//! Parses `#[route("/path")]` attributes on route modules.

use proc_macro_error2::abort;
use proc_macro2::Span;
use syn::Attribute;

/// The extracted path string and metadata from a `#[route("/...")]` attribute.
#[derive(Debug)]
pub struct RouteMacroArgs {
    pub route_ident_span: Span,

    /// A path, defined like: "/" or "/users"
    pub route_path_segments: String,
}

impl RouteMacroArgs {
    /// Scans `attrs` for a `#[route("/...")]` attribute, validates the path
    /// (must start with `/`, must not end with `/` unless root, no `//`),
    /// and returns the parsed args. Returns `None` if no `#[route]` is found.
    pub fn parse(attrs: &[Attribute]) -> Option<RouteMacroArgs> {
        attrs
            .iter()
            .find(|attr| attr.path().is_ident("route"))
            .and_then(|attr| {
                let ident = attr.path().get_ident().unwrap();

                if matches!(attr.meta, syn::Meta::Path(_)) {
                    abort!(ident.span(), "Missing path argument. Use `#[route(\"/path\")]`.");
                }

                match attr.parse_args_with(|input: syn::parse::ParseStream| {
                    let lookahead = input.lookahead1();
                    if !lookahead.peek(syn::LitStr) {
                        abort!(input.span(), "Expected a path string literal, e.g. #[route(\"/users\")].");
                    }
                    let lit: syn::LitStr = input.parse()?;
                    let val = lit.value();
                    if !val.starts_with('/') {
                        abort!(lit.span(), "Every path must start with a '/'. Add a leading '/'.");
                    }
                    if val.ends_with('/') && val.len() > 1 {
                        abort!(lit.span(), "No path should end with a '/'. Remove the trailing '/'.");
                    }
                    if val.contains("//") {
                        abort!(lit.span(), "Separate each part with one '/'. Coalesce consecutive slashes into one.");
                    }

                    if !input.is_empty() {
                        abort!(input.span(), "Unexpected tokens after path. #[route] only takes a path string. Use layout!(), index!(), and page!() body macros inside the module body to declare components.");
                    }

                    Ok(RouteMacroArgs {
                        route_ident_span: ident.span(),
                        route_path_segments: val,
                    })
                }) {
                    Ok(args) => Some(args),
                    Err(e) => abort!(ident.span(), "Failed to parse #[route] arguments: {}", e),
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    fn make_attrs(attr_code: &str) -> Vec<Attribute> {
        let code = format!("{attr_code} fn dummy() {{}}");
        let item: syn::ItemFn = syn::parse_str(&code).unwrap();
        item.attrs
    }

    #[test]
    fn no_route_attribute_returns_none() {
        let attrs = make_attrs("#[other(\"/foo\")]");
        assert_that!(RouteMacroArgs::parse(&attrs)).is_none();
    }

    #[test]
    fn empty_attributes_returns_none() {
        let attrs: Vec<Attribute> = vec![];
        assert_that!(RouteMacroArgs::parse(&attrs)).is_none();
    }

    #[test]
    fn valid_root_path() {
        let attrs = make_attrs("#[route(\"/\")]");
        let result = RouteMacroArgs::parse(&attrs);
        assert_that!(result.as_ref()).is_some();
        assert_that!(result.unwrap().route_path_segments.as_str()).is_equal_to("/");
    }

    #[test]
    fn valid_static_path() {
        let attrs = make_attrs("#[route(\"/users\")]");
        let result = RouteMacroArgs::parse(&attrs);
        assert_that!(result.as_ref()).is_some();
        assert_that!(result.unwrap().route_path_segments.as_str()).is_equal_to("/users");
    }

    #[test]
    fn valid_param_path() {
        let attrs = make_attrs("#[route(\"/:id\")]");
        let result = RouteMacroArgs::parse(&attrs);
        assert_that!(result.as_ref()).is_some();
        assert_that!(result.unwrap().route_path_segments.as_str()).is_equal_to("/:id");
    }

    #[test]
    fn valid_complex_path() {
        let attrs = make_attrs("#[route(\"/users/:id/posts\")]");
        let result = RouteMacroArgs::parse(&attrs);
        assert_that!(result.as_ref()).is_some();
        assert_that!(result.unwrap().route_path_segments.as_str()).is_equal_to("/users/:id/posts");
    }

    #[test]
    fn valid_wildcard_path() {
        let attrs = make_attrs("#[route(\"/*rest\")]");
        let result = RouteMacroArgs::parse(&attrs);
        assert_that!(result.as_ref()).is_some();
        assert_that!(result.unwrap().route_path_segments.as_str()).is_equal_to("/*rest");
    }
}
