from uneedtest import TestCase

from structpath import Structpath


class TestStructpathWrite(TestCase):
    def test_write_simple_path(self):
        path = Structpath.parse("$a.b.c")
        data = {}

        result = path.write(data, 42)

        self.assert_equal(result, {"a": {"b": {"c": 42}}})
        self.assert_equal(data, {"a": {"b": {"c": 42}}})

    def test_write_with_none_data(self):
        path = Structpath.parse("$a.b.c")

        result = path.write(None, 42)

        self.assert_equal(result, {"a": {"b": {"c": 42}}})

    def test_write_with_array_indices(self):
        path = Structpath.parse("$a[0].b[1]")
        data = {}

        result = path.write(data, "test")
        self.assert_equal(result, {"a": [{"b": [None, "test"]}]})

    def test_write_to_existing_structure(self):
        path = Structpath.parse("$a.b.c")
        data = {"a": {"b": {"x": 1}}}

        result = path.write(data, 42)

        self.assert_equal(result, {"a": {"b": {"x": 1, "c": 42}}})

    def test_write_to_existing_array(self):
        path = Structpath.parse("$a[1]")
        data = {"a": [0, 1, 2]}

        result = path.write(data, "replaced")

        self.assert_equal(result, {"a": [0, "replaced", 2]})

    def test_write_to_array_out_of_bounds(self):
        path = Structpath.parse("$a[5]")
        data = {"a": [0, 1]}

        result = path.write(data, "extended")

        self.assert_equal(result, {"a": [0, 1, None, None, None, "extended"]})

    def test_write_with_numeric_keys(self):
        path = Structpath.parse("$123.456")
        data = {}

        result = path.write(data, "numeric keys")

        self.assert_equal(result, {"123": {"456": "numeric keys"}})

    def test_write_with_key_variable(self):
        path = Structpath.parse("$a.#var.c")
        data = {}
        vars = {"var": "b"}

        result = path.write(data, 42, vars)

        self.assert_equal(result, {"a": {"b": {"c": 42}}})

    def test_write_with_index_variable(self):
        path = Structpath.parse("$a[#idx].c")
        data = {}
        vars = {"idx": "2"}

        result = path.write(data, 42, vars)

        self.assert_equal(result, {"a": [None, None, {"c": 42}]})

    def test_write_with_mixed_variables(self):
        path = Structpath.parse("$teams[#idx].members.#name")
        data = {}
        vars = {"idx": "1", "name": "alice"}

        result = path.write(data, "developer", vars)

        self.assert_equal(
            result, {"teams": [None, {"members": {"alice": "developer"}}]}
        )

    def test_write_to_root(self):
        path = Structpath.parse("$")
        data = {}

        result = path.write(data, {"root": "value"})

        self.assert_equal(result, {"root": "value"})

    def test_write_complex_data_types(self):
        path = Structpath.parse("$complex")
        data = {}
        complex_value = {
            "string": "test",
            "number": 42,
            "boolean": True,
            "null": None,
            "array": [1, 2, 3],
            "object": {"a": 1, "b": 2},
        }

        result = path.write(data, complex_value)

        self.assert_equal(result, {"complex": complex_value})

    def test_write_overwrite_existing_type(self):
        path1 = Structpath.parse("$data")
        data1 = {"data": [1, 2, 3]}
        result1 = path1.write(data1, {"key": "value"})
        self.assert_equal(result1, {"data": {"key": "value"}})

        path2 = Structpath.parse("$data")
        data2 = {"data": {"key": "value"}}
        result2 = path2.write(data2, [1, 2, 3])
        self.assert_equal(result2, {"data": [1, 2, 3]})

        path3 = Structpath.parse("$data")
        data3 = {"data": 42}
        result3 = path3.write(data3, {"key": "value"})
        self.assert_equal(result3, {"data": {"key": "value"}})

    def test_write_variables_missing_context(self):
        path = Structpath.parse("$a.#var.c")
        data = {}

        with self.assert_raises(ValueError):
            path.write(data, 42)

    def test_write_variables_missing_variable(self):
        path = Structpath.parse("$a.#var.c")
        data = {}
        vars = {}

        with self.assert_raises(ValueError):
            path.write(data, 42, vars)

    def test_write_index_variable_invalid_value(self):
        path = Structpath.parse("$a[#idx].c")
        data = {}
        vars = {"idx": "not_a_number"}

        with self.assert_raises(ValueError):
            path.write(data, 42, vars)

    def test_write_path_starting_with_index(self):
        path = Structpath.parse("$[0].name")

        result = path.write(None, "item one")

        self.assert_equal(result, [{"name": "item one"}])

        path2 = Structpath.parse("$[2][1].value")
        result2 = path2.write(None, 42)

        expected = [None, None, [None, {"value": 42}]]
        self.assert_equal(result2, expected)

    def test_chained_writes(self):
        data = {}

        path1 = Structpath.parse("$users.alice.role")
        result1 = path1.write(data, "admin")

        path2 = Structpath.parse("$users.bob.role")
        result2 = path2.write(result1, "user")

        path3 = Structpath.parse("$settings.theme")
        result3 = path3.write(result2, "dark")

        expected = {
            "users": {"alice": {"role": "admin"}, "bob": {"role": "user"}},
            "settings": {"theme": "dark"},
        }
        self.assert_equal(result3, expected)
