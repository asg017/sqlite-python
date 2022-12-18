from sqlite_python_extensions import scalar_function, table_function, Row

@scalar_function
def hello(name):
  return f"Hello, {name}!"

@table_function(columns=["value", "progress"])
def series(start, stop):
  for value in range(start, stop+1):
    yield Row(value=value, progress=(value - start) / (stop-start))
