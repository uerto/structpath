"""
Structpath - A Python library for navigating nested data structures.

This library provides a way to define paths into nested data structures,
like those found in JSON, and extract values from these structures using
the defined paths.
"""

from typing import Any, Iterator, TypeVar, overload

T = TypeVar("T")
V = TypeVar("V")

class Structpath:
    """
    A path into a nested data structure.

    Structpath provides a way to define paths into nested data structures,
    and extract or write values using these paths.
    """

    def __init__(self) -> None:
        """
        Create a new empty Structpath object.

        Examples:
            >>> path = Structpath()
            >>> path.push_string_key("users")
            >>> path.push_index(0)
        """
        pass

    @staticmethod
    def parse(path_str: str) -> "Structpath":
        """
        Parse a structpath string into a Structpath object.

        Args:
            path_str: A string representation of a path

        Returns:
            A new Structpath object

        Raises:
            ValueError: If the path string cannot be parsed

        Examples:
            >>> path = Structpath.parse("$users[0].name")
            >>> path.get(data)  # Returns "Alice"
        """
        pass

    def push_key(self, key: str | int) -> None:
        """
        Add a key to the path.

        Args:
            key: The key to add, can be a string or integer

        Examples:
            >>> path = Structpath()
            >>> path.push_key("users")  # String key
            >>> path.push_key(42)       # Integer key
        """
        pass

    def push_index(self, index: int) -> None:
        """
        Add an array index to the path.

        Args:
            index: The array index to add

        Examples:
            >>> path = Structpath()
            >>> path.push_string_key("users")
            >>> path.push_index(0)  # First user
            >>> assert str(path) == "$users.0"
        """
        pass

    def push_key_variable(self, name: str) -> None:
        """
        Add a key variable to the path.

        A key variable is a placeholder for a key that will be provided
        when the path is used to access data.

        Args:
            name: The name of the variable

        Raises:
            ValueError: If the variable name is already used in this path

        Examples:
            >>> path = Structpath()
            >>> path.push_string_key("users")
            >>> path.push_key_variable("userId")
            >>> path.get(data, {"userId": "user1"})  # Get user with ID "user1"
        """
        pass

    def push_index_variable(self, name: str) -> None:
        """
        Add an index variable to the path.

        An index variable is a placeholder for an array index that will be
        provided when the path is used to access data.

        Args:
            name: The name of the variable

        Raises:
            ValueError: If the variable name is already used in this path

        Examples:
            >>> path = Structpath()
            >>> path.push_string_key("users")
            >>> path.push_index_variable("idx")
            >>> path.get(data, {"idx": "0"})  # Get user at index 0
        """
        pass

    @overload
    def get(
        self, data: dict[str, V], vars: dict[str, Any] | None = None
    ) -> V: ...
    @overload
    def get(self, data: list[V], vars: dict[str, Any] | None = None) -> V: ...
    @overload
    def get(self, data: Any) -> Any: ...
    def get(self, data: Any, vars: dict[str, Any] | None = None) -> Any:
        """
        Get a value from data using this path.

        Args:
            data: The data structure to navigate
            vars: Optional dictionary mapping variable names to values

        Returns:
            The value at the path

        Raises:
            KeyError: If the path doesn't exist in the data
            IndexError: If an index doesn't exist in the data
            TypeError: If the path is invalid for the data structure
            ValueError: If a variable in the path is missing from vars

        Examples:
            >>> data = {"users": [{"name": "Alice"}, {"name": "Bob"}]}
            >>> path = Structpath.parse("$users[0].name")
            >>> path.get(data)  # Returns "Alice"
            >>> path = Structpath.parse("$users[#idx].name")
            >>> path.get(data, {"idx": "1"})  # Returns "Bob"
        """
        pass

    def iter(self, data: Any) -> Iterator[tuple[dict[str, str], Any]]:
        """
        Iterate over all possible variable resolutions in the data.

        For a path with variables, this method returns an iterator that
        yields tuples of (variable_values, value) for all possible combinations
        of variable values that lead to valid paths in the data.

        Args:
            data: The data structure to navigate

        Returns:
            An iterator yielding (variable_values, value) tuples

        Examples:
            >>> data = {"users": {"user1": {"name": "Alice"}, "user2": {"name": "Bob"}}}
            >>> path = Structpath.parse("$users.#userId.name")
            >>> for vars, value in path.iter(data):
            ...     print(f"{vars['userId']}: {value}")
            user1: Alice
            user2: Bob
        """
        pass

    @overload
    def write(
        self,
        data: dict[str, Any],
        value: Any | None = None,
        vars: dict[str, Any] | None = None,
    ) -> dict[str, Any]: ...
    @overload
    def write(
        self,
        data: list[Any],
        value: Any | None = None,
        vars: dict[str, Any] | None = None,
    ) -> list[Any]: ...
    def write(
        self,
        data: Any | None = None,
        value: Any | None = None,
        vars: dict[str, Any] | None = None,
    ) -> Any:
        """
        Write a value to a path in the data structure.

        This method creates or updates a value at the specified path in the data.
        If parts of the path don't exist, they will be created.

        Args:
            data: The data structure to modify (optional)
            value: The value to write (optional)
            vars: Optional dictionary mapping variable names to values

        Returns:
            The modified data structure

        Raises:
            TypeError: If the path is invalid for the data structure
            ValueError: If a variable in the path is missing from vars

        Examples:
            >>> data = {"users": []}
            >>> path = Structpath.parse("$users[0].name")
            >>> result = path.write(data, "Alice")
            >>> result
            {'users': [{'name': 'Alice'}]}
            >>> path = Structpath.parse("$users.#userId.name")
            >>> result = path.write(data, "Charlie", {"userId": "user3"})
            >>> result
            {'users': [{'name': 'Alice'}], 'user3': {'name': 'Charlie'}}
        """
        pass

    @staticmethod
    def walk(data: T) -> Iterator[tuple["Structpath", Any]]:
        """
        Walk through all paths in a data structure.

        This method returns an iterator that yields tuples of (Structpath, value)
        for every path in the data structure.

        Args:
            data: The data structure to walk through

        Returns:
            An iterator yielding (path, value) tuples

        Examples:
            >>> data = {"a": [1, 2, {"b": 3}], "c": {"d": 4}}
            >>> for path, value in Structpath.walk(data):
            ...     print(f"{path}: {value}")
            $: {'a': [1, 2, {'b': 3}], 'c': {'d': 4}}
            $a: [1, 2, {'b': 3}]
            $a[0]: 1
            $a[1]: 2
            $a[2]: {'b': 3}
            $a[2].b: 3
            $c: {'d': 4}
            $c.d: 4
        """
        pass

    def __str__(self) -> str:
        """
        Return a string representation of the path.

        Returns:
            A string representation of the path

        Examples:
            >>> path = Structpath.parse("$users[0].name")
            >>> str(path)  # Returns "$users[0].name"
        """
        pass

    def __repr__(self) -> str:
        """
        Return a string representation of the Structpath object.

        Returns:
            A string representation of the Structpath object

        Examples:
            >>> path = Structpath.parse("$users[0].name")
            >>> repr(path)  # Returns "Structpath('$users[0].name')"
        """
        pass

__all__ = ["Structpath"]
