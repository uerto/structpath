# Structpath Variables

Structpath supports variables within path expressions, allowing for dynamic path
resolution at runtime. This is one of the most powerful features of Structpath,
enabling flexible data access patterns.

## Variable Syntax

There are two types of variables in Structpath:

1. **Key Variables**: Used to represent object keys

   - Syntax: `#variable_name`
   - Example: `$users.#userId.profile`

1. **Index Variables**: Used to represent array indices

   - Syntax: `[#variable_name]`
   - Example: `$items[#index].value`

## Resolving Variables

Variables are resolved at runtime by providing a dictionary that maps variable
names to their values. The resolution context is passed as an argument to
methods like `get()`, `write()`, and others.

```python
from structpath import Structpath

data = {
    "users": {
        "user1": {"name": "Alice"},
        "user2": {"name": "Bob"}
    }
}

path = Structpath.parse("$users.#userId.name")

# Resolve using different variable values
alice_name = path.get(data, {"userId": "user1"})
assert alice_name == "Alice"

bob_name = path.get(data, {"userId": "user2"})
assert bob_name == "Bob"
```

## Variable Rules

1. **Uniqueness**: Each variable name must be unique within a path.

   ```python
   # This will raise an error
   try:
      Structpath.parse("$users.#var.items.#var")
      assert False, "Should have raised an error"
   except ValueError as e:
      assert "Duplicate variable" in str(e)
   ```

1. **Required Resolution**: If a path contains variables, a variable context
   must be provided.

   ```python
   path = Structpath.parse("$users.#userId.profile")

   try:
      path.get(data)  # No variable context provided
      assert False
   except ValueError as e:
      assert "Path contains variables" in str(e)
   ```

1. **Type Conversion**: Index variables must be convertible to integers.

   ```python
   data = {"items": ["a", "b", "c"]}
   path = Structpath.parse("$items[#index]")

   # String values that represent integers are converted automatically
   value = path.get(data, {"index": "1"})
   assert value == "b"

   # Non-integer values will cause an error
   try:
      path.get(data, {"index": "not-a-number"})
      assert False, "Should have raised an error"
   except ValueError as e:
      assert "Invalid variable value" in str(e)
   ```

## Use Cases for Variables

### 1. Parameterized Data Access

Variables allow the same path to be reused with different parameters:

```python
from structpath import Structpath

# Configuration data
config = {
    "environments": {
        "dev": {
            "database": {
                "host": "dev-db.example.com",
                "port": 5432
            }
        },
        "prod": {
            "database": {
                "host": "prod-db.example.com",
                "port": 3306
            }
        }
    }
}

# Create a parameterized path
path = Structpath.parse("$environments.#env.database.host")

# Get database host for different environments
dev_host = path.get(config, {"env": "dev"})
assert dev_host == "dev-db.example.com"

prod_host = path.get(config, {"env": "prod"})
assert prod_host == "prod-db.example.com"
```

### 2. Finding All Matches with `iter()`

The `iter()` method allows you to find all possible variable resolutions for a
path. It returns an iterator of (variable_dict, value) tuples:

```python
from structpath import Structpath

# Team membership data
data = {
    "teams": {
        "engineering": {
            "members": {
                "alice": {"role": "developer"},
                "bob": {"role": "designer"}
            }
        },
        "marketing": {
            "members": {
                "charlie": {"role": "manager"},
                "dave": {"role": "copywriter"}
            }
        }
    }
}

# Path with multiple variables
path = Structpath.parse("$teams.#teamName.members.#memberName.role")

# Find all team members and their roles
team_roles = {}
for vars_dict, role in path.iter(data):
    team = vars_dict["teamName"]
    member = vars_dict["memberName"]
    team_roles[(team, member)] = role

# Check the results
assert team_roles[("engineering", "alice")] == "developer"
assert team_roles[("engineering", "bob")] == "designer"
assert team_roles[("marketing", "charlie")] == "manager"
assert team_roles[("marketing", "dave")] == "copywriter"
```

### 3. Dynamic Data Transformation

Variables can be used to transform data based on patterns:

```python
from structpath import Structpath

# Source data
source = {
    "users": [
        {"id": "user1", "name": "Alice", "email": "alice@example.com"},
        {"id": "user2", "name": "Bob", "email": "bob@example.com"}
    ]
}

# Target data structure
target = {}

# Create a path with an index variable
path = Structpath.parse("$users[#idx]")
write_path = Structpath.parse("$userMap.#id")

# Transform the data structure
for vars_dict, user in path.iter(source):
    user_id = user["id"]
    write_path.write(target, user, {"id": user_id})

# Check the result
assert target["userMap"]["user1"]["name"] == "Alice"
assert target["userMap"]["user2"]["email"] == "bob@example.com"
```

### 4. Complex Queries

Variables enable complex query patterns:

```python
from structpath import Structpath

# Product inventory data
inventory = {
    "categories": {
        "electronics": {
            "products": [
                {"id": "e1", "name": "Phone", "price": 499, "stock": 10},
                {"id": "e2", "name": "Laptop", "price": 999, "stock": 5}
            ]
        },
        "books": {
            "products": [
                {"id": "b1", "name": "Python Guide", "price": 29, "stock": 100},
                {"id": "b2", "name": "Data Science", "price": 39, "stock": 50}
            ]
        }
    }
}

# Find all products with low stock (less than 10)
low_stock = []
path = Structpath.parse("$categories.#category.products[#idx]")

for vars_dict, product in path.iter(inventory):
    if product["stock"] < 10:
        low_stock.append({
            "category": vars_dict["category"],
            "product": product["name"],
            "stock": product["stock"]
        })

assert low_stock[0]["category"] == "electronics"
assert low_stock[0]["product"] == "Laptop"
assert low_stock[0]["stock"] == 5
```

## Advanced Variable Usage

### Variable Resolution with Path Building

Variables can also be used when building paths programmatically:

```python
from structpath import Structpath

# Create a path with variables
path = Structpath()
path.push_key("users")
path.push_key_variable("userId")
path.push_key("addresses")
path.push_index_variable("addrIdx")

# Now the path is equivalent to: $users.#userId.addresses[#addrIdx]
assert str(path) == "$users.#userId.addresses[#addrIdx]"

# Use the path
data = {
    "users": {
        "user1": {
            "addresses": [
                {"type": "home", "city": "New York"},
                {"type": "work", "city": "Boston"}
            ]
        }
    }
}

home_city = path.get(data, {"userId": "user1", "addrIdx": "0"})["city"]
assert home_city == "New York"

work_city = path.get(data, {"userId": "user1", "addrIdx": "1"})["city"]
assert work_city == "Boston"
```

### Multiple Variable Patterns

You can use multiple path patterns with different variables to access related data:

```python
from structpath import Structpath

# Organization data
org_data = {
    "departments": {
        "eng": {"name": "Engineering", "budget": 1000000},
        "mkt": {"name": "Marketing", "budget": 500000}
    },
    "employees": {
        "alice": {"department": "eng", "salary": 120000},
        "bob": {"department": "eng", "salary": 110000},
        "charlie": {"department": "mkt", "salary": 90000}
    }
}

# Find employees and their departments
employee_path = Structpath.parse("$employees.#empId")
dept_id_path = Structpath.parse("$employees.#empId.department")
dept_name_path = Structpath.parse("$departments.#deptId.name")

employee_info = {}
for vars_dict, employee in employee_path.iter(org_data):
    emp_id = vars_dict["empId"]

    # Get the employee's department ID
    dept_id = dept_id_path.get(org_data, {"empId": emp_id})

    # Get the department name using the department ID
    dept_name = dept_name_path.get(org_data, {"deptId": dept_id})

    employee_info[emp_id] = {
        "salary": employee["salary"],
        "department": dept_name
    }

assert employee_info["alice"]["department"] == "Engineering"
assert employee_info["charlie"]["department"] == "Marketing"
assert employee_info["bob"]["salary"] == 110000
```
