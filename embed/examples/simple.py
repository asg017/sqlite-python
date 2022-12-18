from sqlite_python_extensions import scalar_function, table_function, Row

@scalar_function
def simple_version():
  return "v0.0.1"

@scalar_function
def simple_reverse(s):
  return str(s)[::-1]

import itertools

@table_function(columns=["value", "x", "y"])
def simple_product(a, b):
  for value in itertools.product(a,b):
    yield Row(value=value, x=value[0], y=value[1])

@table_function(columns=["str", "bool_true", "bool_false", "dict", "list", "tuple"])
def simple_all():
  yield Row(
    str="string",
    bool_true=True,
    bool_false=True,
    dict={"a":1, "b": [2]},
    list=["a", "b", None, 4],
    tuple=(1,2),
  )