from sqlite_python_extensions import scalar_function

@scalar_function
def a():
  return "aaa"

@scalar_function
def aa():
  return "aaaaaa"