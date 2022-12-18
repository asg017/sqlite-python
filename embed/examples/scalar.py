from sqlite_python_extensions import scalar_function

@scalar_function
def scalar_noparams():
  return "yo"

@scalar_function
def scalar_param1(a):
  return f"yo {a}"

@scalar_function
def scalar_paramoptional(a=None):
  return a

@scalar_function
def scalar_param_multiple_optional(a, b=None, c=None):
  return [a, b, c]

#@scalar_function
def scalar_spread(*args):
  return [len(args), args]


#@scalar_function
def scalar_spreada(a, *args):
  return [a, len(args), args]

#@scalar_function
def scalar_spreadb(*args, b):
  return [b, len(args), args]

#@scalar_function
def scalar_spreadall(a, *args, b):
  return [a, b, len(args), args]

@scalar_function
def a():
  return "aaa"
