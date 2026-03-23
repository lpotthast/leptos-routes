//! Parses route path strings (e.g., `"/users/:id"`) into typed segments.

use quote::quote;

/// Metadata about a single dynamic path parameter, used to generate
/// `materialize()` method signatures.
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub is_optional: bool,
    /// The route path string where this parameter was defined (e.g., `"/:id"`).
    /// Used in diagnostic messages for parameter name collisions.
    pub origin_path: String,
}

/// A single segment of a route path string.
#[derive(Debug, PartialEq, Eq)]
pub enum PathSegment {
    /// Literal path segment (e.g., `"users"`).
    Static(String),
    /// Required dynamic segment (`:name`). Name stored without the leading colon.
    Param(String),
    /// Optional dynamic segment (`:name?`). Name stored without colon and question mark.
    OptionalParam(String),
    /// Catch-all segment (`*name`). Name stored without the leading asterisk.
    Wildcard(String),
}

/// Parsed representation of a route's path string as a sequence of typed segments.
#[derive(Debug, PartialEq, Eq)]
pub struct PathSegments {
    pub segments: Vec<PathSegment>,
}

impl PathSegments {
    /// Splits a path string on `/`, skips empty segments, and classifies each
    /// segment by its prefix character (`:` → param, `*` → wildcard, else static).
    pub fn parse(path: &str) -> PathSegments {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if let Some(param) = segment.strip_prefix(':') {
                    if let Some(optional) = param.strip_suffix('?') {
                        PathSegment::OptionalParam(optional.to_string())
                    } else {
                        PathSegment::Param(param.to_string())
                    }
                } else if let Some(wildcard) = segment.strip_prefix('*') {
                    PathSegment::Wildcard(wildcard.to_string())
                } else {
                    PathSegment::Static(segment.to_string())
                }
            })
            .collect();
        PathSegments { segments }
    }

    /// Generates the appropriate tuple-type for these segments.
    pub fn generate_path_type(&self) -> proc_macro2::TokenStream {
        let segment_types = self.segments.iter().map(|segment| match segment {
            PathSegment::Static(_) => quote!(::leptos_router::StaticSegment<&'static str>),
            PathSegment::Param(_) => quote!(::leptos_router::ParamSegment),
            PathSegment::OptionalParam(_) => quote!(::leptos_router::OptionalParamSegment),
            PathSegment::Wildcard(_) => quote!(::leptos_router::WildcardSegment),
        });

        if self.segments.is_empty() {
            quote!(())
        } else {
            quote!((#(#segment_types,)*))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    mod parse_tests {
        use super::*;

        #[test]
        fn root_path() {
            let parsed = PathSegments::parse("/");
            assert_that!(parsed.segments).is_empty();
        }

        #[test]
        fn single_static_segment() {
            let parsed = PathSegments::parse("/users");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Static("users".into())]);
        }

        #[test]
        fn multiple_static_segments() {
            let parsed = PathSegments::parse("/users/posts");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::Static("users".into()),
                PathSegment::Static("posts".into()),
            ]);
        }

        #[test]
        fn required_param() {
            let parsed = PathSegments::parse("/:id");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Param("id".into())]);
        }

        #[test]
        fn optional_param() {
            let parsed = PathSegments::parse("/:id?");
            assert_that!(parsed.segments)
                .contains_exactly([PathSegment::OptionalParam("id".into())]);
        }

        #[test]
        fn wildcard() {
            let parsed = PathSegments::parse("/*rest");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Wildcard("rest".into())]);
        }

        #[test]
        fn mixed_segments() {
            let parsed = PathSegments::parse("/users/:id/posts/:post_id?");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::Static("users".into()),
                PathSegment::Param("id".into()),
                PathSegment::Static("posts".into()),
                PathSegment::OptionalParam("post_id".into()),
            ]);
        }

        #[test]
        fn static_then_wildcard() {
            let parsed = PathSegments::parse("/files/*path");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::Static("files".into()),
                PathSegment::Wildcard("path".into()),
            ]);
        }

        #[test]
        fn multiple_params() {
            let parsed = PathSegments::parse("/:org/:repo/:branch");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::Param("org".into()),
                PathSegment::Param("repo".into()),
                PathSegment::Param("branch".into()),
            ]);
        }

        #[test]
        fn multiple_optional_params() {
            let parsed = PathSegments::parse("/:year?/:month?");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::OptionalParam("year".into()),
                PathSegment::OptionalParam("month".into()),
            ]);
        }

        #[test]
        fn deeply_nested_static() {
            let parsed = PathSegments::parse("/a/b/c/d/e");
            assert_that!(parsed.segments.as_slice()).has_length(5);
            assert_that!(
                parsed
                    .segments
                    .iter()
                    .all(|s| matches!(s, PathSegment::Static(_)))
            )
            .is_true();
        }

        #[test]
        fn single_char_static_segment() {
            let parsed = PathSegments::parse("/a");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Static("a".into())]);
        }

        #[test]
        fn single_char_param() {
            let parsed = PathSegments::parse("/:x");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Param("x".into())]);
        }

        #[test]
        fn wildcard_only() {
            let parsed = PathSegments::parse("/*any");
            assert_that!(parsed.segments).contains_exactly([PathSegment::Wildcard("any".into())]);
        }

        #[test]
        fn param_followed_by_wildcard() {
            let parsed = PathSegments::parse("/:id/*rest");
            assert_that!(parsed.segments).contains_exactly([
                PathSegment::Param("id".into()),
                PathSegment::Wildcard("rest".into()),
            ]);
        }
    }
}
