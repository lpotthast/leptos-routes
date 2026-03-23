//! Tracks the current position in the module nesting hierarchy during route collection.

/// A stack of module identifiers representing a path from the root `#[routes]` module
/// to the current position. Used to determine where generated structs should be inserted
/// and to build qualified paths for referencing them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePath {
    idents: Vec<syn::Ident>,
}

impl ModulePath {
    /// Creates a new path containing only the root module identifier.
    pub fn root(root: syn::Ident) -> Self {
        Self { idents: vec![root] }
    }

    /// Appends a child module identifier to the path.
    pub fn push(&mut self, ident: syn::Ident) {
        self.idents.push(ident);
    }

    /// Returns the intermediate module segments, excluding the root module (first)
    /// and the current module (last).
    ///
    /// For `[routes, users, user, details]` this returns `[users, user]`.
    pub fn intermediate_segments(&self) -> &[syn::Ident] {
        self.idents
            .get(1..self.idents.len().saturating_sub(1))
            .unwrap_or_default()
    }

    /// Returns the number of segments in the path.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.idents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;
    use proc_macro2::Span;

    fn ident(name: &str) -> syn::Ident {
        syn::Ident::new(name, Span::call_site())
    }

    #[test]
    fn root_has_single_element() {
        let path = ModulePath::root(ident("routes"));
        assert_that!(path.len()).is_equal_to(1);
        assert_that!(path.idents).contains_exactly([ident("routes")]);
    }

    #[test]
    fn push_increases_length() {
        let mut path = ModulePath::root(ident("routes"));
        path.push(ident("users"));
        assert_that!(path.len()).is_equal_to(2);
        path.push(ident("user"));
        assert_that!(path.len()).is_equal_to(3);
    }

    mod intermediate_segments {
        use super::*;

        #[test]
        fn on_single_element_returns_empty() {
            // [routes] → intermediate_segments = [] (only root, no current module)
            let path = ModulePath::root(ident("routes"));
            assert_that!(path.intermediate_segments()).is_empty();
        }

        #[test]
        fn on_two_elements_returns_empty() {
            // [routes, users] → intermediate_segments = [] (excludes first and last)
            let mut path = ModulePath::root(ident("routes"));
            path.push(ident("users"));
            assert_that!(path.intermediate_segments()).is_empty();
        }

        #[test]
        fn on_three_elements_returns_middle() {
            // [routes, users, details] → intermediate_segments = [users]
            let mut path = ModulePath::root(ident("routes"));
            path.push(ident("users"));
            path.push(ident("details"));
            let middle = path.intermediate_segments();
            assert_that!(middle).contains_exactly([ident("users")]);
        }

        #[test]
        fn on_four_elements_returns_middle() {
            // [routes, users, user, details] → intermediate_segments = [users, user]
            let mut path = ModulePath::root(ident("routes"));
            path.push(ident("users"));
            path.push(ident("user"));
            path.push(ident("details"));
            let middle = path.intermediate_segments();
            assert_that!(middle).contains_exactly([ident("users"), ident("user")]);
        }
    }

    #[test]
    fn equality() {
        let mut a = ModulePath::root(ident("routes"));
        a.push(ident("users"));
        let mut b = ModulePath::root(ident("routes"));
        b.push(ident("users"));
        assert_that!(a).is_equal_to(b);
    }

    #[test]
    fn inequality() {
        let mut a = ModulePath::root(ident("routes"));
        a.push(ident("users"));
        let mut b = ModulePath::root(ident("routes"));
        b.push(ident("posts"));
        assert_that!(a).is_not_equal_to(b);
    }
}
