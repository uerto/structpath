# uerto-structpath

`uerto-structpath`is a Python library for querying, traversing, and manipulating
structured data using path expressions.

A Structpath is a path expression that allows to navigate through nested
data structures. It's similar to
[JSONPath](https://goessner.net/articles/JsonPath/) or
[XPath](https://developer.mozilla.org/en-US/docs/Web/XPath). By design, it is
less powerful, as it does not allow for complex queries. It is more powerful in
a different regard, as it allows defining variables in a path.

A typical Structpath looks like this:

```general
$users.admin.permissions[2].level
```

Structpath uses a simple syntax:

- `$` represents the root of the document
- `.` separates path segments for object properties
- `[n]` accesses array elements
- `#varname` defines variables that can be resolved at runtime

## Installation

```shell
pip install structpath
```

## Example

```python
from structpath import Structpath

data = {
    "users": {
        "admin": {"permissions": ["read", "write", "execute"]},
        "guest": {"permissions": ["read"]}
    }
}

# Parse a path and get a value
path = Structpath.parse("$users.admin.permissions[0]")
value = path.get(data)
assert value == "read"

# Use variables in paths
path_with_var = Structpath.parse("$users.#role.permissions[0]")
value = path_with_var.get(data, {"role": "guest"})
assert value == "read"

# Write to a nested location
path.write(data, "read-only")
assert data["users"]["admin"]["permissions"][0] == "read-only"

# Walk through data
for path_str, value in Structpath.walk(data):
    # Process each path and value
    pass
```
