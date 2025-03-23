from uneedtest import TestCase

from structpath import Structpath


class TestStructpathParse(TestCase):
    def test_parse_simple_path(self):
        path = Structpath.parse("$user.name")
        self.assert_equal(str(path), "$user.name")

        path = Structpath.parse("$products[1].name")
        self.assert_equal(str(path), "$products[1].name")

    def test_roundtrip(self):
        paths = [
            "$user.name",
            "$products[1].price",
            r"$special\.key",
            r"$user.settings.notifications\.enabled",
            "$123",
            r"$\123",
        ]

        for path_str in paths:
            path = Structpath.parse(path_str)
            new_path_str = str(path)
            new_path = Structpath.parse(new_path_str)
            self.assert_equal(str(new_path), new_path_str)
            # tests/test_structpath.py

    def test_parse_with_variables(self):
        # Test parsing paths with variables
        path = Structpath.parse("$a.#var.c")
        self.assertEqual(str(path), "$a.#var.c")

        # Test with array access
        path = Structpath.parse("$a[0].#var[1]")
        self.assertEqual(str(path), "$a[0].#var[1]")

        # Test with escaped characters
        path = Structpath.parse(r"$a\.#var\[0\].c")
        self.assertEqual(str(path), "$a\\.\\#var\\[0\\].c")
