use crate::types::{Structpath, StructpathError};

pub fn parse(path_str: &str) -> Result<Structpath, StructpathError> {
    let mut path = Structpath::new();
    let mut chars = path_str.chars().peekable();

    if chars.peek() == Some(&'$') {
        chars.next();
    }

    let mut current_segment = String::new();
    let mut in_brackets = false;
    let mut escape_next = false;
    let mut is_escaped_segment = false;
    let mut first_char_escaped = false;
    let mut is_variable = false;

    for c in chars {
        if escape_next {
            current_segment.push(c);
            escape_next = false;

            if current_segment.len() == 1 {
                first_char_escaped = true;
            }
            is_escaped_segment = true;
            continue;
        }

        match c {
            '\\' => {
                escape_next = true;
            }
            '.' if !in_brackets => {
                if !current_segment.is_empty() {
                    process_segment(
                        &mut path,
                        &current_segment,
                        first_char_escaped,
                        is_escaped_segment,
                        is_variable,
                        in_brackets,
                    )?;
                    current_segment = String::new();
                    first_char_escaped = false;
                    is_escaped_segment = false;
                    is_variable = false;
                }
            }
            '[' if !in_brackets => {
                if !current_segment.is_empty() {
                    process_segment(
                        &mut path,
                        &current_segment,
                        first_char_escaped,
                        is_escaped_segment,
                        is_variable,
                        in_brackets,
                    )?;
                    current_segment = String::new();
                    first_char_escaped = false;
                    is_escaped_segment = false;
                    is_variable = false;
                }
                in_brackets = true;
            }
            ']' if in_brackets => {
                in_brackets = false;

                if current_segment.starts_with('#') && current_segment.len() > 1
                {
                    let var_name = &current_segment[1..];
                    path.push_index_variable(var_name)?;
                } else if let Ok(index) = current_segment.parse::<usize>() {
                    path.push_index(index);
                } else {
                    return Err(StructpathError::ParseError(format!(
                        "Invalid index: {}",
                        current_segment
                    )));
                }

                current_segment = String::new();
                first_char_escaped = false;
                is_escaped_segment = false;
                is_variable = false;
            }
            '#' if current_segment.is_empty() && !in_brackets => {
                is_variable = true;
                current_segment.push(c);
            }
            _ => current_segment.push(c),
        }
    }

    if !current_segment.is_empty() {
        process_segment(
            &mut path,
            &current_segment,
            first_char_escaped,
            is_escaped_segment,
            is_variable,
            in_brackets,
        )?;
    }

    if in_brackets {
        return Err(StructpathError::ParseError(
            "Unclosed bracket".to_string(),
        ));
    }

    Ok(path)
}

fn process_segment(
    path: &mut Structpath,
    segment: &str,
    first_char_escaped: bool,
    is_escaped_segment: bool,
    is_variable: bool,
    in_brackets: bool,
) -> Result<(), StructpathError> {
    if is_variable && segment.starts_with('#') && segment.len() > 1 {
        // Variable segment: extract variable name (remove leading #)
        let var_name = &segment[1..];

        if in_brackets {
            path.push_index_variable(var_name)?;
        } else {
            path.push_key_variable(var_name)?;
        }

        return Ok(());
    }
    if first_char_escaped || is_escaped_segment {
        path.push_string_key(segment);
        return Ok(());
    }
    if let Ok(int_key) = segment.parse::<i64>() {
        path.push_int_key(int_key);
        return Ok(());
    }
    path.push_string_key(segment);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_path() {
        let path = parse("$a.b.c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_string_key("b");
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_key_variables() {
        let path = parse("$a.#var.c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_key_variable("var");
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_index_variables() {
        let path = parse("$a[#idx].b").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_index_variable("idx");
        let _ = expected.push_string_key("b");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_multiple_variable_types() {
        let path = parse("$teams[#idx].members.#name").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("teams");
        let _ = expected.push_index_variable("idx");
        let _ = expected.push_string_key("members");
        let _ = expected.push_key_variable("name");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_variables() {
        let path = parse("$a.#var.c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_key_variable("var");
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_array_variable() {
        let path = parse("$a[#idx].b").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_index_variable("idx");
        let _ = expected.push_string_key("b");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_multiple_unique_variables() {
        let path = parse("$teams.#teamId.members.#userId").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("teams");
        let _ = expected.push_key_variable("teamId");
        let _ = expected.push_string_key("members");
        let _ = expected.push_key_variable("userId");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_duplicate_variables() {
        let result = parse("$teams.#var.members.#var");

        assert!(matches!(result, Err(StructpathError::DuplicateVariable(_))));
    }

    #[test]
    fn test_parse_with_array_indices() {
        let path = parse("$a[0].b[1].c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_index(0);
        let _ = expected.push_string_key("b");
        let _ = expected.push_index(1);
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_escaped_characters() {
        let path = parse(r"$a\.b\[0\].c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a.b[0]");
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_int_keys() {
        let path = parse("$123.456.789").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_int_key(123);
        let _ = expected.push_int_key(456);
        let _ = expected.push_int_key(789);

        assert_eq!(path, expected);
    }

    #[test]
    fn test_parse_with_escaped_hash() {
        let path = parse(r"$a.\#notvar.c").unwrap();

        let mut expected = Structpath::new();
        let _ = expected.push_string_key("a");
        let _ = expected.push_string_key("#notvar");
        let _ = expected.push_string_key("c");

        assert_eq!(path, expected);
    }
}
