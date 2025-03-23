from datetime import datetime

from uneedtest import TestCase


class _CustomClass:
    def __init__(self, value):
        self.value = value


class StructpathTestCase(TestCase):
    def set_up(self):
        self.test_data = {
            "user": {
                "name": "John Doe",
                "age": 30,
                "addresses": [
                    {
                        "type": "home",
                        "street": "123 Main St",
                        "city": "Anytown",
                    },
                    {
                        "type": "work",
                        "street": "456 Oak Ave",
                        "city": "Somewhere",
                    },
                ],
                "settings": {"theme": "dark", "notifications.enabled": True},
            },
            "products": [
                {"id": 1, "name": "Widget", "price": 19.99},
                {"id": 2, "name": "Gadget", "price": 29.99},
                {"id": 3, "name": "Doohickey", "price": 14.99},
            ],
            "123": "integer key",
            "special.key": "key with dot",
            "data": datetime(2024, 12, 24),
        }
