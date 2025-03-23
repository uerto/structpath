use crate::types::{Segment, SegmentKey, Structpath};

pub fn to_string(path: &Structpath) -> String {
    if path.segments().is_empty() {
        return "$".to_string();
    }

    let mut result = String::from("$");
    let mut first = true;

    for segment in path.segments() {
        match segment {
            Segment::Key(key) => {
                format_key_segment(&mut result, key, &mut first);
            }
            Segment::Index(idx) => {
                result.push_str(&format!("[{}]", idx));
            }
            Segment::KeyVariable(var_name) => {
                format_key_variable(&mut result, var_name, &mut first);
            }
            Segment::IndexVariable(var_name) => {
                format_index_variable(&mut result, var_name);
            }
        }
    }

    result
}

fn format_key_segment(result: &mut String, key: &SegmentKey, first: &mut bool) {
    match key {
        SegmentKey::String(string_key) => {
            format_string_key(result, string_key, first);
        }
        SegmentKey::Int(int_key) => {
            format_int_key(result, *int_key, first);
        }
    }
}

fn format_string_key(result: &mut String, string_key: &str, first: &mut bool) {
    if *first {
        *first = false;
    } else {
        result.push('.');
    }

    if string_key.parse::<i64>().is_ok() {
        result.push('\\');
    }

    result.push_str(&escape_special_chars(string_key));
}

fn format_int_key(result: &mut String, int_key: i64, first: &mut bool) {
    if *first {
        *first = false;
    } else {
        result.push('.');
    }

    result.push_str(&int_key.to_string());
}

fn format_key_variable(result: &mut String, var_name: &str, first: &mut bool) {
    if *first {
        *first = false;
    } else {
        result.push('.');
    }

    // Format key variable with leading #
    result.push('#');
    result.push_str(var_name);
}

fn format_index_variable(result: &mut String, var_name: &str) {
    // Format index variable with [#name]
    result.push_str(&format!("[#{}]", var_name));
}

fn escape_special_chars(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '.' | '[' | ']' | '\\' | '#' => format!("\\{}", c), // Also escape # character
            _ => c.to_string(),
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn test_to_string() {
        let mut path = Structpath::new();
        path.push_string_key("a");
        path.push_index(0);
        path.push_string_key("b.c");
        path.push_string_key("d[e]");
        path.push_string_key("123");
        path.push_int_key(456);

        // Use the function directly
        let path_str = to_string(&path);
        assert_eq!(path_str, r"$a[0].b\.c.d\[e\].\123.456");
    }

    #[test]
    fn test_with_key_variable() {
        let mut path = Structpath::new();
        let _ = path.push_string_key("a");
        let _ = path.push_key_variable("var");
        let _ = path.push_string_key("c");

        let path_str = to_string(&path);
        assert_eq!(path_str, "$a.#var.c");
    }

    #[test]
    fn test_with_index_variable() {
        let mut path = Structpath::new();
        let _ = path.push_string_key("a");
        let _ = path.push_index_variable("idx");
        let _ = path.push_string_key("b");

        let path_str = to_string(&path);
        assert_eq!(path_str, "$a[#idx].b");
    }

    #[test]
    fn test_with_mixed_variables() {
        let mut path = Structpath::new();
        let _ = path.push_string_key("teams");
        let _ = path.push_index_variable("idx");
        let _ = path.push_string_key("members");
        let _ = path.push_key_variable("name");

        let path_str = to_string(&path);
        assert_eq!(path_str, "$teams[#idx].members.#name");
    }

    #[test]
    fn test_with_hash_in_key() {
        let mut path = Structpath::new();
        let _ = path.push_string_key("a");
        let _ = path.push_string_key("#notvar"); // Regular key with hash character
        let _ = path.push_string_key("c");

        let path_str = to_string(&path);
        assert_eq!(path_str, r"$a.\#notvar.c");
    }

    #[test]
    fn test_roundtrip() {
        let path_strs = vec![
            "$a.b.c",
            "$a[0].b[1].c",
            r"$a\.b.c",
            r"$a.b\[0\].c",
            r"$a\.b\.c",
            r"$123.456.abc",
            r"$\123.\456.abc",
            "$a.#var.c",
            "$teams.#teamId.members.#userId",
            "$items[#idx].value",
        ];

        for path_str in path_strs {
            let path = parse::parse(path_str).unwrap();
            // Use Display trait implementation via format! macro
            let new_path_str = format!("{}", path);
            let new_path = parse::parse(&new_path_str).unwrap();
            assert_eq!(path, new_path);
        }
    }
}
