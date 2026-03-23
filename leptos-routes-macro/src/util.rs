//! String manipulation utilities for identifier generation in code output.

/// Converts a `snake_case` string to `PascalCase`.
///
/// Used to derive struct names from module names (e.g., `user_details` → `UserDetails`).
///
/// Note: non-initial characters within each word are lowercased. This means
/// all-uppercase inputs lose their casing (e.g., `"URL"` → `"Url"`,
/// `"API"` → `"Api"`). This is intentional for consistent struct naming.
pub fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }

    result
}

/// Appends `_` to identifiers that collide with Rust keywords, so they can be
/// used as parameter names in generated `materialize()` signatures
/// (e.g., `:type` (path segment) → `type_` (parameter name).
pub fn sanitize_identifier(name: &str) -> String {
    const RUST_KEYWORDS: &[&str] = &[
        // Strict keywords
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", // Reserved keywords
        "abstract", "become", "box", "do", "final", "gen", "macro", "override", "priv", "try",
        "typeof", "unsized", "virtual", "yield",
    ];

    if RUST_KEYWORDS.contains(&name) {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assertr::prelude::*;

    mod to_pascal_case {
        use super::*;

        #[test]
        fn single_word() {
            assert_that!(to_pascal_case("users")).is_equal_to("Users");
        }

        #[test]
        fn snake_case() {
            assert_that!(to_pascal_case("user_details")).is_equal_to("UserDetails");
        }

        #[test]
        fn multiple_underscores() {
            assert_that!(to_pascal_case("my_long_module_name")).is_equal_to("MyLongModuleName");
        }

        #[test]
        fn already_capitalized() {
            assert_that!(to_pascal_case("Users")).is_equal_to("Users");
        }

        #[test]
        fn all_caps_lowercases_after_first() {
            assert_that!(to_pascal_case("URL")).is_equal_to("Url");
        }

        #[test]
        fn empty_string() {
            assert_that!(to_pascal_case("")).is_equal_to("");
        }

        #[test]
        fn leading_underscore() {
            assert_that!(to_pascal_case("_private")).is_equal_to("Private");
        }

        #[test]
        fn trailing_underscore() {
            assert_that!(to_pascal_case("item_")).is_equal_to("Item");
        }

        #[test]
        fn consecutive_underscores() {
            assert_that!(to_pascal_case("a__b")).is_equal_to("AB");
        }

        #[test]
        fn single_char() {
            assert_that!(to_pascal_case("a")).is_equal_to("A");
        }
    }

    mod sanitize_identifier {
        use super::*;

        #[test]
        fn non_keyword_unchanged() {
            assert_that!(sanitize_identifier("name")).is_equal_to("name");
        }

        #[test]
        fn strict_keyword_gets_suffix() {
            assert_that!(sanitize_identifier("type")).is_equal_to("type_");
        }

        // Special case: Only stric keyword not fully lowercase.
        #[test]
        fn upper_self_keyword() {
            assert_that!(sanitize_identifier("Self")).is_equal_to("Self_");
        }

        #[test]
        fn all_strict_keywords_are_sanitized() {
            let all_strict_keywords = [
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
                "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct",
                "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ];
            for keyword in all_strict_keywords {
                assert_that!(sanitize_identifier(keyword)).ends_with("_");
            }
        }

        #[test]
        fn reserved_keyword_gets_suffix() {
            assert_that!(sanitize_identifier("gen")).is_equal_to("gen_");
        }

        #[test]
        fn all_reserved_keywords_are_sanitized() {
            let all_reserved_keywords = [
                "abstract", "become", "box", "do", "final", "gen", "macro", "override", "priv",
                "try", "typeof", "unsized", "virtual", "yield",
            ];
            for keyword in all_reserved_keywords {
                assert_that!(sanitize_identifier(keyword)).ends_with("_");
            }
        }

        #[test]
        fn keyword_prefix_not_affected() {
            // "types" is not a keyword, even though "type" is
            assert_that!(sanitize_identifier("types")).is_equal_to("types");
        }
    }
}
