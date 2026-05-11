import os
import unused_module
from typing import TYPE_CHECKING

# Mutable default
def foo(items=[]):
    eval("1+1")
    x = 1
    return 2

# Missing docstring
class Bar:
    # Shadow builtin
    def process(list):
        if list == None:
            return True
        else:
            return False

# Bare except
try:
    foo()
except:
    pass
