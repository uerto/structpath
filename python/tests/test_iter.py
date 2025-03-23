from base import StructpathTestCase

from structpath import Structpath


class TestStructpathIter(StructpathTestCase):
    def test_iter_with_variables(self):
        data = {
            "users": {
                "user1": {"name": "Alice", "score": 85},
                "user2": {"name": "Bob", "score": 92},
            }
        }
        path = Structpath.parse("$users.#userId.score")
        results = list(path.iter(data))
        self.assertEqual(len(results), 2)
        self.assertTrue(
            any(
                value == 85 and vars_dict["userId"] == "user1"
                for vars_dict, value in results
            )
        )
        self.assertTrue(
            any(
                value == 92 and vars_dict["userId"] == "user2"
                for vars_dict, value in results
            )
        )

    def test_multiple_variables_iter(self):
        data = {
            "teams": {
                "team1": {"members": {"user1": 85, "user2": 92}},
                "team2": {"members": {"user3": 78, "user4": 88}},
            }
        }
        path = Structpath.parse("$teams.#teamId.members.#userId")
        results = list(path.iter(data))
        self.assertEqual(len(results), 4)
        expected_results = [
            ({"teamId": "team1", "userId": "user1"}, 85),
            ({"teamId": "team1", "userId": "user2"}, 92),
            ({"teamId": "team2", "userId": "user3"}, 78),
            ({"teamId": "team2", "userId": "user4"}, 88),
        ]
        for expected_vars, expected_val in expected_results:
            self.assertTrue(
                any(
                    value == expected_val and vars_dict == expected_vars
                    for vars_dict, value in results
                )
            )

    def test_iter_with_array_variables(self):
        data = {
            "items": [
                {"id": "item1", "tags": ["red", "large"]},
                {"id": "item2", "tags": ["blue", "small"]},
            ]
        }
        path = Structpath.parse("$items[#idx].id")
        results = list(path.iter(data))
        self.assertEqual(len(results), 2)
        self.assertTrue(
            any(
                value == "item1" and vars_dict["idx"] == 0
                for vars_dict, value in results
            )
        )
        self.assertTrue(
            any(
                value == "item2" and vars_dict["idx"] == 1
                for vars_dict, value in results
            )
        )

    def test_error_on_variable_without_context(self):
        data = {"a": {"b": {"c": 42}}}
        path = Structpath.parse("$a.#key.c")
        with self.assertRaises(ValueError):
            path.get(data)
