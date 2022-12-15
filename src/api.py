from inspect import signature, Parameter, _empty

class ScalarFunction():
  def __init__(self, function):
    argc = 0
    
    for name, param in signature(function).parameters.items():
      if param.kind == param.VAR_KEYWORD or param.kind == param.VAR_POSITIONAL:
        raise Exception
      argc += 1
    
    self.argc = argc
    self.function = function
    self.scalar_function = True

class Column:
  def __init__(self, name, hidden=False, required=False):
    self.name = name
    self.hidden = hidden
    self.required = hidden

class TableFunction:
  def __init__(self, function, row):
    hidden_columns_parameters = []
    hidden_columns_other = []
    columns = []

    for name, param in signature(function).parameters.items():
      if param.kind == param.VAR_KEYWORD or param.kind == param.VAR_POSITIONAL:
        raise Exception
      if param.default is param.empty:
        hidden_columns_parameters.append(name)
    
    for name in row.columns:
      columns.append(Column(name))
    for name in hidden_columns_parameters:
      columns.append(Column(name, hidden=True))

    sql = 'create table x('
    for i, column in enumerate(columns):
      if i != 0:
        sql += ','
      if column.hidden:
        sql += f"{column.name} hidden"
      else:
        sql += column.name
    sql += ')'
    print(sql)

    self.name = function.__name__
    self.sql = sql
    self.columns = columns
    self.generator = function
    self.table_function = True


def scalar_function(function):
  #def wrapped():
  #  return function()
  #wrapped.scalar_function = True
  return ScalarFunction(function)


def table_function(row, innocuous=None):
  def decorator(function):
    return TableFunction(function, row)
  return decorator

"""
Decorator
"""
def column(func):
  def wrapper(self):
    return func(self)
  wrapper.is_column = True
  return wrapper

"""
Decorator
"""
def row(cls):
  cls.columns = []
  for name, method in cls.__dict__.items():
    if hasattr(method, "is_column"):
      cls.columns.append(name)
  return cls
