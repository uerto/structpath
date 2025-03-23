from collections.abc import Iterator
from typing import Any, Dict

from uneedtest import TestCase

from structpath import Structpath


class TestStructpathWalk(TestCase):
    def test_walk_is_iterator(self):
        data = [{1: 1}]
        self.assert_is_instance(Structpath.walk(data), Iterator)

    def test_walk(self):
        simple_data = {"a": {"b": 1, "c": [2, 3, {"d": 4}]}}

        results = Structpath.walk(simple_data)

        value_map = {str(p): v for p, v in results}
        self.assert_equal(value_map["$a.b"], 1)
        self.assert_equal(value_map["$a.c[0]"], 2)
        self.assert_equal(value_map["$a.c[1]"], 3)
        self.assert_equal(value_map["$a.c[2].d"], 4)

    def test_walk_with_scalar(self):
        """Test walker with a scalar value."""
        data = 42
        walker = Structpath.walk(data)

        # Should have just one value (the root)
        path, value = next(walker)
        self.assert_equal(str(path), "$")
        self.assert_equal(value, 42)

        # No more values
        self.assert_raises(StopIteration, lambda: next(walker))

    def test_walk_with_array(self):
        """Test walker with a simple array."""
        data = [1, 2, 3]
        results = list(Structpath.walk(data))

        # Should have 4 results: the array itself and its 3 elements
        self.assert_equal(len(results), 4)

        # Convert to paths and values for easier testing
        paths = [str(path) for path, _ in results]
        values = [value for _, value in results]

        # Check paths
        self.assert_in("$", paths)
        self.assert_in("$[0]", paths)
        self.assert_in("$[1]", paths)
        self.assert_in("$[2]", paths)

        # Check values
        self.assert_in([1, 2, 3], values)
        self.assert_in(1, values)
        self.assert_in(2, values)
        self.assert_in(3, values)

    def test_walk_with_object(self):
        """Test walker with a simple object."""
        data = {"a": 1, "b": 2}
        results = list(Structpath.walk(data))

        # Should have 3 results: the object itself and its 2 properties
        self.assert_equal(len(results), 3)

        # Convert to paths and values
        paths = [str(path) for path, _ in results]
        values = [value for _, value in results]

        # Check paths
        self.assert_in("$", paths)
        self.assert_in("$a", paths)
        self.assert_in("$b", paths)

        # Check values
        self.assert_in({"a": 1, "b": 2}, values)
        self.assert_in(1, values)
        self.assert_in(2, values)

    def test_walk_with_nested_structure(self):
        """Test walker with a nested object and array structure."""
        data = {
            "users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}],
            "metadata": {"version": "1.0", "created": "2023-01-01"},
        }

        results = list(Structpath.walk(data))

        # Check some specific paths
        paths = [str(path) for path, _ in results]

        # Root
        self.assert_in("$", paths)

        # First level properties
        self.assert_in("$users", paths)
        self.assert_in("$metadata", paths)

        # Array elements
        self.assert_in("$users[0]", paths)
        self.assert_in("$users[1]", paths)

        # Nested object properties
        self.assert_in("$users[0].name", paths)
        self.assert_in("$users[0].age", paths)
        self.assert_in("$users[1].name", paths)
        self.assert_in("$users[1].age", paths)
        self.assert_in("$metadata.version", paths)
        self.assert_in("$metadata.created", paths)

        # Verify some values
        path_value_map: Dict[str, Any] = {
            str(path): value for path, value in results
        }

        self.assert_equal(path_value_map["$users[0].name"], "Alice")
        self.assert_equal(path_value_map["$users[1].age"], 25)
        self.assert_equal(path_value_map["$metadata.version"], "1.0")

    def test_walk_with_int_keys(self):
        """Test walker with object that has integer keys."""
        data = {123: "integer key value", "normal": "string key value"}

        results = list(Structpath.walk(data))

        # Check for int key path
        paths = [str(path) for path, _ in results]

        # Should create the correct path format
        self.assert_in("$123", paths)
        self.assert_in("$normal", paths)

        # Check values
        path_value_map = {str(path): value for path, value in results}

        self.assert_equal(path_value_map["$123"], "integer key value")

    def test_walk_with_empty_structures(self):
        """Test walker with empty objects and arrays."""
        # Empty object
        data = {}
        results = list(Structpath.walk(data))

        # Should just have the root object
        self.assert_equal(len(results), 1)
        self.assert_equal(str(results[0][0]), "$")

        # Empty array
        data = []
        results = list(Structpath.walk(data))

        # Should just have the root array
        self.assert_equal(len(results), 1)
        self.assert_equal(str(results[0][0]), "$")

    def test_walk_depth_first_traversal(self):
        """Test that the walker uses depth-first traversal."""
        data = {"a": {"b": {"c": 1}}}

        results = list(Structpath.walk(data))

        # Convert to paths for easier comparison
        paths = [str(path) for path, _ in results]

        # Find indices for the paths
        root_idx = paths.index("$")
        a_idx = paths.index("$a")
        a_b_idx = paths.index("$a.b")
        a_b_c_idx = paths.index("$a.b.c")

        # In depth-first order, the deepest nodes should come first,
        # followed by their parents
        self.assert_true(a_b_c_idx < a_b_idx)
        self.assert_true(a_b_idx < a_idx)
        self.assert_true(a_idx < root_idx)

    def test_walk_with_complex_nested_structure(self):
        """Test walker with a complex nested structure."""
        data = {
            "products": [
                {
                    "id": "p1",
                    "name": "Product 1",
                    "variants": [
                        {"color": "red", "size": "S"},
                        {"color": "blue", "size": "M"},
                    ],
                },
                {
                    "id": "p2",
                    "name": "Product 2",
                    "variants": [{"color": "green", "size": "L"}],
                },
            ],
            "stats": {
                "total": 2,
                "categories": {"clothing": 1, "accessories": 1},
            },
        }

        results = list(Structpath.walk(data))

        # Check for some specific deep paths
        paths = [str(path) for path, _ in results]

        self.assert_in("$products[0].variants[0].color", paths)
        self.assert_in("$products[0].variants[1].size", paths)
        self.assert_in("$products[1].variants[0].color", paths)
        self.assert_in("$stats.categories.clothing", paths)

        # Verify a few values
        path_value_map = {str(path): value for path, value in results}

        self.assert_equal(
            path_value_map["$products[0].variants[1].color"], "blue"
        )
        self.assert_equal(path_value_map["$stats.categories.accessories"], 1)

    def test_walk_with_none_values(self):
        """Test walker with None/null values."""
        data = {"a": None, "b": [None, {"c": None}]}

        results = list(Structpath.walk(data))

        # Check paths and values
        path_value_map = {str(path): value for path, value in results}

        self.assert_equal(path_value_map["$a"], None)
        self.assert_equal(path_value_map["$b[0]"], None)
        self.assert_equal(path_value_map["$b[1].c"], None)

    def test_walk_with_bool_values(self):
        """Test walker with boolean values."""
        data = {"truthy": True, "falsy": False}

        results = list(Structpath.walk(data))

        # Check paths and values
        path_value_map = {str(path): value for path, value in results}

        self.assert_equal(path_value_map["$truthy"], True)
        self.assert_equal(path_value_map["$falsy"], False)
