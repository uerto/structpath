use serde_json::json;
use structpath::Structpath;

#[test]
fn test_full_api() {
    let path1 = Structpath::parse("$users[0].name").unwrap();

    let mut path2 = Structpath::new();
    path2.push_string_key("users");
    path2.push_index(0);
    path2.push_string_key("name");A

    assert_eq!(path1, path2);
    assert_eq!(format!("{}", path1), "$users[0].name");

    let data = json!({
        "users": [
            {"name": "Alice", "email": "alice@example.com"},
            {"name": "Bob", "email": "bob@example.com"}
        ]
    });

    let value = path1.get(&data, None).unwrap();
    assert_eq!(*value, json!("Alice"));

    let path3 = Structpath::parse(r"$special\.key.\123").unwrap();
    let data2 = json!({
        "special.key": {
            "123": "found"
        }
    });

    let value2 = path3.get(&data2, None).unwrap();
    assert_eq!(*value2, json!("found"));

    // Test walking
    let walk_data = json!({
        "a": [1, {"b": 2}],
        "c": 3
    });

    let results = Structpath::walk(&walk_data);
    let paths: Vec<String> =
        results.map(|(path, _)| format!("{}", path)).collect();

    assert!(paths.contains(&"$".to_string()));
    assert!(paths.contains(&"$a".to_string()));
    assert!(paths.contains(&"$a[0]".to_string()));
    assert!(paths.contains(&"$a[1]".to_string()));
    assert!(paths.contains(&"$a[1].b".to_string()));
    assert!(paths.contains(&"$c".to_string()));
}

#[test]
fn test_numeric_keys() {
    // Test with numeric keys (both as string and int)
    let mut path1 = Structpath::new();
    path1.push_int_key(123);
    path1.push_string_key("456"); // String key that looks like a number

    let path_str = format!("{}", path1);
    assert_eq!(path_str, r"$123.\456");

    let path2 = Structpath::parse(&path_str).unwrap();
    assert_eq!(path2, path1);

    let data = json!({
        "123": {
            "456": "value"
        }
    });

    let value = path1.get(&data, None).unwrap();
    assert_eq!(*value, json!("value"));
}

#[test]
fn test_error_handling() {
    let path = Structpath::parse("$a.b[0].c").unwrap();

    // Test NotFound error
    let data1 = json!({"a": {"x": 1}});
    let result1 = path.get(&data1, None);
    assert!(matches!(
        result1,
        Err(structpath::StructpathError::NotFound)
    ));

    // Test InvalidPath error
    let data2 = json!({"a": 1});
    let result2 = path.get(&data2, None);
    assert!(matches!(
        result2,
        Err(structpath::StructpathError::InvalidPath { .. })
    ));

    // Test IndexOutOfBounds error
    let data3 = json!({"a": {"b": []}});
    let result3 = path.get(&data3, None);
    assert!(matches!(
        result3,
        Err(structpath::StructpathError::IndexOutOfBounds(_))
    ));

    // Test ParseError
    let result4 = Structpath::parse("$a[unclosed");
    assert!(matches!(
        result4,
        Err(structpath::StructpathError::ParseError(_))
    ));
}

#[test]
fn test_display_trait() {
    // Test that the Display trait works correctly
    let path = Structpath::parse("$users[0].name").unwrap();

    // Different ways to use Display
    let s1 = format!("{}", path);
    let s2 = path.to_string(); // from ToString trait, implemented via Display

    assert_eq!(s1, "$users[0].name");
    assert_eq!(s2, "$users[0].name");
}

#[test]
fn test_roundtrip_preservation() {
    // Test that parsing and to_string maintain the same semantics
    let test_paths = vec![
        "$a.b.c",
        "$a[0].b.c",
        "$123.456",
        r"$\123.\456",
        r"$a\.b\.c",
        r"$a[0].b\[0\].c",
    ];

    for path_str in test_paths {
        let path1 = Structpath::parse(path_str).unwrap();
        let new_str = format!("{}", path1);
        let path2 = Structpath::parse(&new_str).unwrap();

        assert_eq!(path1, path2, "Roundtrip failed for path: {}", path_str);
    }
}
