"""Module docstring, reworded."""

import functools


# The cache keeps files in memory.
@functools.lru_cache(maxsize=None)
def load(path):
    """Load a file, reworded.

    With a second paragraph.
    """
    with open(path) as f:  # a trailing comment
        data = f.read()
    if data:
        data = data.strip()
        return data
    return ""


class Store:
    '''A store, reworded.'''

    limit = 10  # the limit

    def get(self, key, default=None):
        value = self.items.get(key, default)
        # a new comment
        return value

    class Meta:
        """Added docstring."""

        def version(self):
            """Added docstring."""
            return "1  0"


def helper(x):
    """Added docstring."""
    y = x - 2
    return y
