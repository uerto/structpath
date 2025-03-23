use crate::types::{Segment, Structpath};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

/// A state item for the variable iterator
#[derive(Clone)]
struct VariableIterState<'a> {
    value: &'a Value,
    current_segment_idx: usize,
    variable_values: HashMap<String, Value>,
}

/// An iterator that finds all possible variable resolutions for a path in a data structure
pub struct VariableIterator<'a> {
    stack: VecDeque<VariableIterState<'a>>,
    path: &'a Structpath,
    visited: HashSet<String>, // Track visited paths to avoid duplicates
}

impl<'a> VariableIterator<'a> {
    /// Create a new iterator to find all possible variable resolutions for a path
    pub fn new(path: &'a Structpath, data: &'a Value) -> Self {
        let mut stack = VecDeque::new();

        // Initial state with empty path and variable values
        stack.push_back(VariableIterState {
            value: data,
            current_segment_idx: 0,
            variable_values: HashMap::new(),
        });

        VariableIterator {
            stack,
            path,
            visited: HashSet::new(),
        }
    }
}

impl<'a> Iterator for VariableIterator<'a> {
    type Item = (&'a Value, HashMap<String, Value>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(state) = self.stack.pop_front() {
            // If we've processed all segments, we found a match
            if state.current_segment_idx >= self.path.segments().len() {
                // Create a unique key for this result to avoid duplicates
                let key = format!("{:?}", state.variable_values);
                if !self.visited.insert(key) {
                    continue; // Skip if we've already seen this combination
                }

                return Some((state.value, state.variable_values));
            }

            // Get the current segment to process
            let current_segment =
                &self.path.segments()[state.current_segment_idx];

            match current_segment {
                Segment::Key(key_segment) => {
                    // Try to navigate to the next level using the key
                    match key_segment {
                        crate::types::SegmentKey::String(key) => {
                            if let Value::Object(map) = state.value {
                                if let Some(next_value) = map.get(key) {
                                    let mut new_state = state.clone();
                                    new_state.value = next_value;
                                    new_state.current_segment_idx += 1;
                                    self.stack.push_back(new_state);
                                }
                            }
                        }
                        crate::types::SegmentKey::Int(key) => {
                            let key_str = key.to_string();
                            if let Value::Object(map) = state.value {
                                if let Some(next_value) = map.get(&key_str) {
                                    let mut new_state = state.clone();
                                    new_state.value = next_value;
                                    new_state.current_segment_idx += 1;
                                    self.stack.push_back(new_state);
                                }
                            }
                        }
                    }
                }
                Segment::Index(idx) => {
                    // Try to navigate to the next level using the array index
                    if let Value::Array(arr) = state.value {
                        if let Some(next_value) = arr.get(*idx) {
                            let mut new_state = state.clone();
                            new_state.value = next_value;
                            new_state.current_segment_idx += 1;
                            self.stack.push_back(new_state);
                        }
                    }
                }
                Segment::KeyVariable(var_name) => {
                    // Handle key variable
                    if let Value::Object(map) = state.value {
                        // Try all object keys as possible values for the variable
                        for (key, next_value) in map {
                            let mut new_state = state.clone();
                            // Store key as a string Value
                            new_state.variable_values.insert(
                                var_name.clone(),
                                Value::String(key.clone()),
                            );
                            new_state.value = next_value;
                            new_state.current_segment_idx += 1;
                            self.stack.push_back(new_state);
                        }
                    }
                }
                Segment::IndexVariable(var_name) => {
                    // Handle index variable
                    if let Value::Array(arr) = state.value {
                        // Try all array indices as possible values for the variable
                        for (idx, next_value) in arr.iter().enumerate() {
                            let mut new_state = state.clone();
                            // Store index as a number Value
                            new_state.variable_values.insert(
                                var_name.clone(),
                                Value::Number(serde_json::Number::from(
                                    idx as u64,
                                )),
                            );
                            new_state.value = next_value;
                            new_state.current_segment_idx += 1;
                            self.stack.push_back(new_state);
                        }
                    }
                }
            }
        }

        None
    }
}

/// Create a VariableIterator for all possible variable resolutions in a path
pub fn iter_variables<'a>(
    path: &'a Structpath,
    data: &'a Value,
) -> VariableIterator<'a> {
    VariableIterator::new(path, data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;
    use serde_json::json;

    #[test]
    fn test_iter_variables() {
        let data = json!({
            "users": {
                "user1": {"name": "Alice", "score": 85},
                "user2": {"name": "Bob", "score": 92}
            }
        });

        // Create a path with a variable
        let path = parse("$users.#userId.score").unwrap();

        // Get all matching values with their variable resolutions
        let results: Vec<_> = iter_variables(&path, &data).collect();

        // Should find 2 matches, one for each user
        assert_eq!(results.len(), 2);

        // Check if we have the expected variable resolutions and values
        let mut found_user1 = false;
        let mut found_user2 = false;

        for (value, vars) in &results {
            if let Some(Value::String(user_id)) = vars.get("userId") {
                match user_id.as_str() {
                    "user1" => {
                        assert_eq!(**value, json!(85));
                        found_user1 = true;
                    }
                    "user2" => {
                        assert_eq!(**value, json!(92));
                        found_user2 = true;
                    }
                    _ => panic!("Unexpected userId: {}", user_id),
                }
            }
        }

        assert!(found_user1, "Did not find user1 in results");
        assert!(found_user2, "Did not find user2 in results");
    }

    #[test]
    fn test_iter_with_multiple_variables() {
        let data = json!({
            "teams": {
                "team1": {
                    "members": {
                        "user1": 85,
                        "user2": 92
                    }
                },
                "team2": {
                    "members": {
                        "user3": 78,
                        "user4": 88
                    }
                }
            }
        });

        // Create a path with multiple variables
        let path = parse("$teams.#teamId.members.#userId").unwrap();

        // Get all matches
        let results: Vec<_> = iter_variables(&path, &data).collect();

        // Should find 4 combinations (2 teams × 2 users per team)
        assert_eq!(results.len(), 4);

        // Define expected results
        let expected_results = [
            (json!(85), "team1", "user1"),
            (json!(92), "team1", "user2"),
            (json!(78), "team2", "user3"),
            (json!(88), "team2", "user4"),
        ];

        // Check all expected combinations are found
        for (expected_value, expected_team, expected_user) in &expected_results
        {
            let found = results.iter().any(|(value, vars)| {
                **value == *expected_value
                    && vars.get("teamId")
                        == Some(&Value::String(expected_team.to_string()))
                    && vars.get("userId")
                        == Some(&Value::String(expected_user.to_string()))
            });

            assert!(
                found,
                "Missing expected result: value={}, teamId={}, userId={}",
                expected_value, expected_team, expected_user
            );
        }
    }

    #[test]
    fn test_iter_with_index_variables() {
        let data = json!({
            "items": [
                {"id": "item1", "tags": ["red", "large"]},
                {"id": "item2", "tags": ["blue", "small"]}
            ]
        });

        // Path with array index as variable
        let path = parse("$items[#idx].id").unwrap();

        // Get all matches
        let results: Vec<_> = iter_variables(&path, &data).collect();

        // Should find 2 matches
        assert_eq!(results.len(), 2);

        // Check if both items are found with correct indices (as integers)
        let item1_found = results.iter().any(|(value, vars)| {
            **value == json!("item1")
                && vars.get("idx") == Some(&Value::Number(0.into()))
        });

        let item2_found = results.iter().any(|(value, vars)| {
            **value == json!("item2")
                && vars.get("idx") == Some(&Value::Number(1.into()))
        });

        assert!(item1_found, "Did not find item1 with idx=0");
        assert!(item2_found, "Did not find item2 with idx=1");
    }

    #[test]
    fn test_iter_with_mixed_variable_types() {
        let data = json!({
            "teams": [
                {
                    "name": "Team A",
                    "members": {
                        "user1": "Alice",
                        "user2": "Bob"
                    }
                },
                {
                    "name": "Team B",
                    "members": {
                        "user3": "Charlie",
                        "user4": "Dave"
                    }
                }
            ]
        });

        // Mix of index and key variables
        let path = parse("$teams[#teamIdx].members.#userId").unwrap();

        // Get all matches
        let results: Vec<_> = iter_variables(&path, &data).collect();

        // Should find 4 matches (2 teams × 2 users per team)
        assert_eq!(results.len(), 4);

        // Check some expected combinations
        let alice_found = results.iter().any(|(value, vars)| {
            **value == json!("Alice")
                && vars.get("teamIdx") == Some(&Value::Number(0.into()))
                && vars.get("userId")
                    == Some(&Value::String("user1".to_string()))
        });

        let dave_found = results.iter().any(|(value, vars)| {
            **value == json!("Dave")
                && vars.get("teamIdx") == Some(&Value::Number(1.into()))
                && vars.get("userId")
                    == Some(&Value::String("user4".to_string()))
        });

        assert!(
            alice_found,
            "Did not find Alice with teamIdx=0, userId=user1"
        );
        assert!(dave_found, "Did not find Dave with teamIdx=1, userId=user4");
    }
}
