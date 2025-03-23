from base import StructpathTestCase

from structpath import Structpath


class TestStructpathGet(StructpathTestCase):
    def test_get_value(self):
        path = Structpath.parse("$user.name")
        self.assert_equal(path.get(self.test_data), "John Doe")

        path = Structpath.parse("$user.age")
        self.assert_equal(path.get(self.test_data), 30)

        path = Structpath.parse("$products[1].price")
        self.assert_equal(path.get(self.test_data), 29.99)

        path = Structpath.parse("$user.addresses[0].street")
        self.assert_equal(path.get(self.test_data), "123 Main St")

    def test_escaped_keys(self):
        path = Structpath.parse(r"$special\.key")
        self.assert_equal(path.get(self.test_data), "key with dot")

        path = Structpath.parse(r"$user.settings.notifications\.enabled")
        self.assert_equal(path.get(self.test_data), True)

    def test_integer_keys(self):
        path = Structpath.parse("$123")
        self.assert_equal(path.get(self.test_data), "integer key")

        escaped_path = Structpath.parse(r"$\123")
        self.assert_equal(str(escaped_path), r"$\123")

    def test_doc(self):
        some_path = Structpath.parse("$users[0].name")
        data = {"users": [{"name": "John Doe", "email": "john@example.com"}]}

        self.assert_equal(some_path.get(data), "John Doe")

    def test_get(self):
        data = {"a": {"b": {"c": 42}, "x": {"c": 100}}}

        # Create a path with a variable
        path = Structpath.parse("$a.#key.c")

        # Resolve with different variable values
        self.assertEqual(path.get(data, {"key": "b"}), 42)
        self.assertEqual(path.get(data, {"key": "x"}), 100)

        # Test missing variable
        with self.assertRaises(ValueError):
            path.get(data, {})

        # Test invalid variable value
        with self.assertRaises(KeyError):
            path.get(data, {"key": "z"})

    def test_error_on_variable_without_context(self):
        data = {"a": {"b": {"c": 42}}}

        # Create a path with a variable
        path = Structpath.parse("$a.#key.c")

        # Using get with a path containing variables should error
        with self.assertRaises(ValueError):
            path.get(data)
