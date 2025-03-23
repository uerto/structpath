from base import StructpathTestCase

from structpath import Structpath


class TestStructpathErrors(StructpathTestCase):
    def test_parse_errors(self):
        with self.assertRaises(ValueError):
            Structpath.parse("$user[unclosed")

        with self.assertRaises(ValueError):
            Structpath.parse("$user[abc]")  # Non-numeric index

    def test_get_errors(self):
        path = Structpath.parse("$user.nonexistent")
        with self.assertRaises(KeyError):
            path.get(self.test_data)

        path = Structpath.parse("$products[10]")
        with self.assertRaises(IndexError):
            path.get(self.test_data)

        path = Structpath.parse(
            "$user.name[0]"
        )  # String doesn't support indexing
        with self.assertRaises(TypeError):
            path.get(self.test_data)
