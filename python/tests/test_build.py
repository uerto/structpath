from base import StructpathTestCase

from structpath import Structpath


class TestStructpathBuild(StructpathTestCase):
    def test_build_path_programmatically(self):
        path = Structpath()
        path.push_key("user")
        path.push_key("addresses")
        path.push_index(1)
        path.push_key("city")

        self.assert_equal(str(path), "$user.addresses[1].city")
        self.assert_equal(path.get(self.test_data), "Somewhere")

        path = Structpath()
        path.push_key(123)
        self.assert_equal(str(path), r"$123")
        self.assert_equal(path.get(self.test_data), "integer key")

    def test_build_with_variables(self):
        path = Structpath()
        path.push_key("teams")
        path.push_key_variable("teamId")
        path.push_key("members")
        path.push_key_variable("userId")

        self.assertEqual(str(path), "$teams.#teamId.members.#userId")

        data = {"teams": {"team1": {"members": {"user1": 85, "user2": 92}}}}

        value = path.get(data, {"teamId": "team1", "userId": "user2"})
        self.assertEqual(value, 92)

    def test_build_with_duplicate_variables(self):
        path = Structpath()
        path.push_key("users")
        path.push_key_variable("id")
        path.push_key("posts")

        # This should still be valid (we're only checking at parse time)
        self.assertEqual(str(path), "$users.#id.posts")

        # Add another variable with the same name
        with self.assertRaises(ValueError) as context:
            path.push_index_variable("id")

        self.assertTrue("Duplicate variable name" in str(context.exception))

    def test_parse_time_duplicate_detection(self):
        # Attempting to parse a path with duplicate variables should fail
        with self.assertRaises(ValueError) as context:
            Structpath.parse("$users.#var.posts.#var")

        self.assertTrue("Duplicate variable name" in str(context.exception))

    def test_direct_variable_usage(self):
        # Directly using path with variables using get() without context should fail
        path = Structpath()
        path.push_key("users")
        path.push_key_variable("id")
        path.push_key("score")

        data = {"users": {"user1": {"score": 85}}}

        # Using get with a path containing variables should error
        with self.assertRaises(ValueError) as context:
            path.get(data)

        self.assertTrue("Path contains variables" in str(context.exception))

        value = path.get(data, {"id": "user1"})
        self.assertEqual(value, 85)
