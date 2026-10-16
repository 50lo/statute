"""Module docstring."""

import functools
import os


@functools.lru_cache(maxsize=None)
def load(path):
    """Load a file."""
    # read it
    with open(path) as f:
        data = f.read()
    if data:
        data = data.strip()
        return data
    return ""


def other():
    return os.sep


class Store:
    """A store."""

    limit = 10

    def get(self, key, default=None):
        """Get a key."""
        value = self.items.get(key, default)
        return value

    class Meta:
        def version(self):
            return "1  0"


def helper(x):
    y = x - 3
    return y
