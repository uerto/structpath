use crate::types::Structpath;
use serde_json::Value;
use std::collections::VecDeque;

pub fn new_walker(data: &Value) -> impl Iterator<Item = (Structpath, &Value)> {
    Walker::new(&Structpath::new(), data)
}

/// A state item for the Walker's traversal stack
#[derive(Clone)]
struct WalkerItem<'a> {
    path: Structpath,
    value: &'a Value,
    processed: bool,
}

/// An iterator that walks through a JSON-like data structure depth-first
pub struct Walker<'a> {
    stack: VecDeque<WalkerItem<'a>>,
}

impl<'a> Walker<'a> {
    /// Create a new Walker to iterate over the data starting from the given path
    pub fn new(_path: &Structpath, data: &'a Value) -> Self {
        let mut stack = VecDeque::new();
        stack.push_back(WalkerItem {
            path: Structpath::new(),
            value: data,
            processed: false,
        });
        Walker { stack }
    }
}

impl<'a> Iterator for Walker<'a> {
    type Item = (Structpath, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut item) = self.stack.pop_front() {
            if !item.processed {
                // Mark as processed and push back to the stack
                item.processed = true;

                // Process children before revisiting this node
                match item.value {
                    Value::Object(map) => {
                        // Push the current item back to the stack to be returned later
                        self.stack.push_front(item.clone());

                        // Then push all children to be processed first (in reverse order)
                        let mut entries: Vec<_> = map.iter().collect();
                        // Reverse to maintain expected traversal order
                        entries.reverse();

                        for (key, value) in entries {
                            let mut new_path = item.path.clone();
                            if let Ok(int_key) = key.parse::<i64>() {
                                new_path.push_int_key(int_key);
                            } else {
                                new_path.push_string_key(key);
                            }

                            self.stack.push_front(WalkerItem {
                                path: new_path,
                                value,
                                processed: false,
                            });
                        }
                    }
                    Value::Array(arr) => {
                        // Push the current item back to the stack to be returned later
                        self.stack.push_front(item.clone());

                        // Then push all array items to be processed first (in reverse order)
                        for (idx, value) in arr.iter().enumerate().rev() {
                            let mut new_path = item.path.clone();
                            new_path.push_index(idx);

                            self.stack.push_front(WalkerItem {
                                path: new_path,
                                value,
                                processed: false,
                            });
                        }
                    }
                    _ => {
                        // For scalar values, just return the item directly
                        return Some((item.path, item.value));
                    }
                }

                // Get the next item
                return self.next();
            } else {
                // Item has been processed, return it
                return Some((item.path, item.value));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_walker_with_scalar() {
        let data = json!(42);
        let mut walker = new_walker(&data);

        // Should have just one value (the root)
        let result = walker.next().unwrap();
        assert_eq!(format!("{}", result.0), "$");
        assert_eq!(*result.1, json!(42));

        // No more values
        assert!(walker.next().is_none());
    }

    #[test]
    fn test_walker_with_array() {
        let data = json!([1, 2, 3]);
        let walker = new_walker(&data);

        // Collect all results
        let results: Vec<_> = walker.collect();

        // Should have 4 results: the array itself and its 3 elements
        assert_eq!(results.len(), 4);

        // Check paths and values
        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        let values: Vec<&Value> =
            results.iter().map(|(_, value)| *value).collect();

        assert!(paths.contains(&"$".to_string()));
        assert!(paths.contains(&"$[0]".to_string()));
        assert!(paths.contains(&"$[1]".to_string()));
        assert!(paths.contains(&"$[2]".to_string()));

        assert!(values.contains(&&json!([1, 2, 3])));
        assert!(values.contains(&&json!(1)));
        assert!(values.contains(&&json!(2)));
        assert!(values.contains(&&json!(3)));
    }

    #[test]
    fn test_walker_with_object() {
        let data = json!({"a": 1, "b": 2});
        let walker = new_walker(&data);

        // Collect all results
        let results: Vec<_> = walker.collect();

        // Should have 3 results: the object itself and its 2 properties
        assert_eq!(results.len(), 3);

        // Check paths and values
        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        let values: Vec<&Value> =
            results.iter().map(|(_, value)| *value).collect();

        assert!(paths.contains(&"$".to_string()));
        assert!(paths.contains(&"$a".to_string()));
        assert!(paths.contains(&"$b".to_string()));

        assert!(values.contains(&&json!({"a": 1, "b": 2})));
        assert!(values.contains(&&json!(1)));
        assert!(values.contains(&&json!(2)));
    }

    #[test]
    fn test_walker_with_nested_structure() {
        let data = json!({
            "users": [
                {"name": "Alice", "age": 30},
                {"name": "Bob", "age": 25}
            ],
            "metadata": {
                "version": "1.0",
                "created": "2023-01-01"
            }
        });

        let walker = new_walker(&data);
        let results: Vec<_> = walker.collect();

        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        // Root
        assert!(paths.contains(&"$".to_string()));

        // First level properties
        assert!(paths.contains(&"$users".to_string()));
        assert!(paths.contains(&"$metadata".to_string()));

        // Array elements
        assert!(paths.contains(&"$users[0]".to_string()));
        assert!(paths.contains(&"$users[1]".to_string()));

        // Nested object properties
        assert!(paths.contains(&"$users[0].name".to_string()));
        assert!(paths.contains(&"$users[0].age".to_string()));
        assert!(paths.contains(&"$users[1].name".to_string()));
        assert!(paths.contains(&"$users[1].age".to_string()));
        assert!(paths.contains(&"$metadata.version".to_string()));
        assert!(paths.contains(&"$metadata.created".to_string()));

        // Verify some values
        let path_value_map: std::collections::HashMap<String, &Value> = results
            .iter()
            .map(|(path, value)| (format!("{}", path), *value))
            .collect();

        assert_eq!(
            path_value_map.get("$users[0].name").unwrap(),
            &&json!("Alice")
        );
        assert_eq!(path_value_map.get("$users[1].age").unwrap(), &&json!(25));
        assert_eq!(
            path_value_map.get("$metadata.version").unwrap(),
            &&json!("1.0")
        );
    }

    #[test]
    fn test_walker_with_int_keys() {
        let data = json!({
            "123": "integer key value",
            "normal": "string key value"
        });

        let walker = new_walker(&data);
        let results: Vec<_> = walker.collect();

        // Check for int key path
        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        // Should create the correct path format
        assert!(paths.contains(&"$123".to_string()));
        assert!(paths.contains(&"$normal".to_string()));

        // Check values
        let path_value_map: std::collections::HashMap<String, &Value> = results
            .iter()
            .map(|(path, value)| (format!("{}", path), *value))
            .collect();

        assert_eq!(
            path_value_map.get("$123").unwrap(),
            &&json!("integer key value")
        );
    }

    #[test]
    fn test_walker_with_empty_structures() {
        // Empty object
        let data = json!({});
        let walker = new_walker(&data);
        let results: Vec<_> = walker.collect();

        // Should just have the root object
        assert_eq!(results.len(), 1);
        assert_eq!(format!("{}", results[0].0), "$");

        // Empty array
        let data = json!([]);
        let walker = new_walker(&data);
        let results: Vec<_> = walker.collect();

        // Should just have the root array
        assert_eq!(results.len(), 1);
        assert_eq!(format!("{}", results[0].0), "$");
    }

    #[test]
    fn test_walker_depth_first_traversal() {
        let data = json!({
            "a": {
                "b": {
                    "c": 1
                }
            }
        });

        let walker = new_walker(&data);

        // In depth-first traversal, we should visit deeper nodes
        // before returning to parent nodes
        let results: Vec<_> = walker.collect();

        // Convert to paths for easier comparison
        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        // Find indices for the paths
        let root_idx = paths.iter().position(|p| p == "$").unwrap();
        let a_idx = paths.iter().position(|p| p == "$a").unwrap();
        let a_b_idx = paths.iter().position(|p| p == "$a.b").unwrap();
        let a_b_c_idx = paths.iter().position(|p| p == "$a.b.c").unwrap();

        // In depth-first order, the deepest nodes should come first,
        // followed by their parents
        assert!(a_b_c_idx < a_b_idx);
        assert!(a_b_idx < a_idx);
        assert!(a_idx < root_idx);
    }

    #[test]
    fn test_walker_with_complex_nested_structure() {
        let data = json!({
            "products": [
                {
                    "id": "p1",
                    "name": "Product 1",
                    "variants": [
                        {"color": "red", "size": "S"},
                        {"color": "blue", "size": "M"}
                    ]
                },
                {
                    "id": "p2",
                    "name": "Product 2",
                    "variants": [
                        {"color": "green", "size": "L"}
                    ]
                }
            ],
            "stats": {
                "total": 2,
                "categories": {
                    "clothing": 1,
                    "accessories": 1
                }
            }
        });

        let walker = new_walker(&data);
        let results: Vec<_> = walker.collect();

        // We won't count exactly, but verify we find some deeply nested paths
        let paths: Vec<String> = results
            .iter()
            .map(|(path, _)| format!("{}", path))
            .collect();

        // Check for some specific deep paths
        assert!(paths.contains(&"$products[0].variants[0].color".to_string()));
        assert!(paths.contains(&"$products[0].variants[1].size".to_string()));
        assert!(paths.contains(&"$products[1].variants[0].color".to_string()));
        assert!(paths.contains(&"$stats.categories.clothing".to_string()));

        // Verify a few values
        let path_value_map: std::collections::HashMap<String, &Value> = results
            .iter()
            .map(|(path, value)| (format!("{}", path), *value))
            .collect();

        assert_eq!(
            path_value_map
                .get("$products[0].variants[1].color")
                .unwrap(),
            &&json!("blue")
        );
        assert_eq!(
            path_value_map.get("$stats.categories.accessories").unwrap(),
            &&json!(1)
        );
    }
}
