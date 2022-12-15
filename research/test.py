class X:
  a = 1

print(X(1).a)
exit()


from inspect import signature, Parameter, _empty



def scalar_function(function):
  def wrapped():
    function()
  return wrapped

class TableFunction:
  def __init__(self, decorated_func):
    hidden_columns_parameters = []
    hidden_columns_other = []
    columns = []

    f = decorated_func()
    for name, param in signature(f).parameters.items():
      if param.kind == param.VAR_KEYWORD or param.kind == param.VAR_POSITIONAL:
        raise Exception
      if param.default is param.empty:
        hidden_columns_parameters.append(name)
    
    for column in decorated_func.row.columns:
      columns.append(column)
    for column in hidden_columns_parameters:
      columns.append(f"{column} hidden")

    sql = 'create table x('
    for i, column in enumerate(columns):
      if i != 0:
        sql += ','
      sql += column
    sql += ')'
    self.sql = sql
    self.generator = f

def table_function(row, innocuous=None):
  def decorator(function):
    def wrapped():
      return function
    wrapped.row = row
    return wrapped
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


@row
class SeriesRow:
  def __init__(self, value):
      self._value = value
  
  @column 
  def value(self):
    return self._value
  
@table_function(SeriesRow)
def series(start, stop):
  for x in range(start, stop):
    yield SeriesRow(x)

@scalar_function
def echo(p):
  p

def init(register):
  return [echo, series]

tf = TableFunction(series)
print(tf.sql)
print(tf.generator)

exit()


def x():
  for x in [1,2,3,4]:
    print('before', x)
    yield x
    print('after', x)

it = x()
print(it)
print('a', next(it))
print('b', next(it))
print('c', next(it))
print('d', next(it))
print('e', next(it))

exit()

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

@row
class XRow():
  def __init__(self, x):
    self._x = x
  
  @column
  def width(self):
    return self._x * 2
  
  @column
  def height(self):
    return self._x * 3

print(XRow.columns)



exit()

import pdfplumber

def pdfplumber_version():
  return pdfplumber.__version__

def pdfplumber_page_text(page):
  return page.extract_text()

#print(pdfplumber_page_text)
#print(pdfplumber_page_text.__code__.co_argcount)

from inspect import signature, Parameter
for name, parameter in signature(pdfplumber_page_text).parameters.items():
  print(pdfplumber_page_text.__name__, name, parameter.kind)

exit()

print(Row(1))
print(Row(1).width)
print(Row(1))

print(vars(Row))

print('\n-----\n')


#print(ow.height)

import pdfplumber
with pdfplumber.open('test.pdf') as pdf:
  for page in pdf.pages:
    pass#print(page.extract_text())

def abc(first, second, p1=None, p2=4):
  pass

for name, parameter in signature(abc).parameters.items():
  print(name, parameter.kind, parameter.default == _empty)

print('\n========================\n')
def xyz(*args, **kwargs):
  pass

for name, parameter in signature(xyz).parameters.items():
  print(name, parameter.kind, parameter.default)

print('\n========================\n')
def jkl(a, *args):
  pass

for name, parameter in signature(jkl).parameters.items():
  print(name, parameter.kind, parameter.default)

