"""Tests verifying type annotations of the Structpath class."""

from typing import Any, Iterator

from structpath import Structpath


def test_initialization_type() -> None:
    path: Structpath = Structpath()


def test_parse_type() -> None:
    path_str: str = "$users[0].name"
    path: Structpath = Structpath.parse(path_str)


def test_get_method_types() -> None:
    path = Structpath()

    dict_data: dict[str, str] = {"name": "Alice"}
    dict_result: str = path.get(dict_data)

    vars_dict: dict[str, str] = {"userId": "user1"}
    dict_vars_result: str = path.get(dict_data, vars_dict)

    list_data: list[int] = [1, 2, 3]
    list_result: int = path.get(list_data)

    list_vars_result: int = path.get(list_data, vars_dict)

    any_data: Any = {"complex": [1, {"nested": True}]}
    any_result: Any = path.get(any_data)

    any_vars_result: Any = path.get(any_data, vars_dict)


def test_iter_method_type() -> None:
    path = Structpath()
    data: dict[str, Any] = {"users": {"user1": {"name": "Alice"}}}

    iter_result: Iterator[tuple[dict[str, str], Any]] = path.iter(data)

    for vars_dict, value in iter_result:
        var_key: str = list(vars_dict.keys())[0]
        var_value: str = vars_dict[var_key]
        any_value: Any = value


def test_write_method_types() -> None:
    path = Structpath()

    dict_data: dict[str, Any] = {"users": []}
    dict_write_result: dict[str, Any] = path.write(dict_data)

    value: str = "Alice"
    dict_value_write_result: dict[str, Any] = path.write(dict_data, value)

    vars_dict: dict[str, str] = {"userId": "user1"}
    dict_value_vars_write_result: dict[str, Any] = path.write(
        dict_data, value, vars_dict
    )

    list_data: list[Any] = [1, 2, 3]
    list_write_result: list[Any] = path.write(list_data)
    list_value_write_result: list[Any] = path.write(list_data, value)

    list_value_vars_write_result: list[Any] = path.write(
        list_data, value, vars_dict
    )

    generic_data: dict[str, int] = {"count": 42}
    T_write_result: dict[str, int] = path.write(generic_data)
    T_value_write_result: dict[str, int] = path.write(generic_data, value)
    T_value_vars_write_result: dict[str, int] = path.write(
        generic_data, value, vars_dict
    )


def test_walk_method_type() -> None:
    data: dict[str, Any] = {"a": 1, "b": [2, 3]}

    walk_result: Iterator[tuple[Structpath, Any]] = Structpath.walk(data)
    for path, value in walk_result:
        path_str: str = str(path)
        any_value: Any = value


def test_string_representation_types() -> None:
    path = Structpath()

    str_result: str = str(path)
    repr_result: str = repr(path)
