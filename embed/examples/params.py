from sqlite_python_extensions import scalar_function

x = 1

@scalar_function
def get_x():
  return x

@scalar_function
def set_x(value):
  global x
  x = value
  return x

pass