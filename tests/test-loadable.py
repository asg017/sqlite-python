import sqlite3
import unittest
import time
import os

EXT_PATH="./target/debug/libpy0"

def connect(ext):
  db = sqlite3.connect(":memory:")

  db.execute("create table base_functions as select name from pragma_function_list")
  db.execute("create table base_modules as select name from pragma_module_list")

  db.enable_load_extension(True)
  db.load_extension(ext)

  db.execute("create temp table loaded_functions as select name from pragma_function_list where name not in (select name from base_functions) order by name")
  db.execute("create temp table loaded_modules as select name from pragma_module_list where name not in (select name from base_modules) order by name")

  db.row_factory = sqlite3.Row
  return db


db = connect(EXT_PATH)

def explain_query_plan(sql):
  return db.execute("explain query plan " + sql).fetchone()["detail"]

def execute_all(sql, args=None):
  if args is None: args = []
  results = db.execute(sql, args).fetchall()
  return list(map(lambda x: dict(x), results))

FUNCTIONS = [
  'py_attr',
  'py_call',
  'py_call_method',
  'py_callable',
  'py_debug',
  'py_dict',
  'py_eval',
  'py_fmt',
  'py_format',
  'py_function_from_module',
  'py_list',
  'py_module',
  'py_set',
  'py_str',
  'py_tuple', 
  'py_value',
  'py_version'
]

MODULES = [
  "py_each",
  'py_functions',
]
def spread_args(args):                                                          
  return ",".join(['?'] * len(args))

class TestPy(unittest.TestCase):
  def test_funcs(self):
    funcs = list(map(lambda a: a[0], db.execute("select name from loaded_functions").fetchall()))
    self.assertEqual(funcs, FUNCTIONS)

  def test_modules(self):
    modules = list(map(lambda a: a[0], db.execute("select name from loaded_modules").fetchall()))
    self.assertEqual(modules, MODULES)
    
  def test_py_version(self):
    version = 'v0.1.0'
    self.assertEqual(db.execute("select py_version()").fetchone()[0], version)
  
  def test_py_debug(self):
    debug = db.execute("select py_debug()").fetchone()[0]
    self.assertEqual(len(debug.splitlines()), 4)
  
  def test_py_eval(self):
    py_eval = lambda pattern: db.execute("select py_eval(?)", [pattern]).fetchone()[0]
    self.assertEqual(py_eval('1 + 2'), 3)
    self.assertEqual(py_eval('"a" + "b"'), "ab")
    self.assertEqual(py_eval('b"yo"'), b"yo")
    self.assertEqual(py_eval('.1 + .02'), 0.12000000000000001)
    self.assertEqual(py_eval('None'), None)
    self.assertEqual(py_eval('{}'), None)
    self.assertEqual(py_eval('(1,)'), None)
    self.assertEqual(py_eval('[1]'), None)
    
  def test_py_str(self):
    py_str = lambda code: db.execute("select py_str(py_eval(?))", [code]).fetchone()[0]
    self.assertEqual(py_str('"a"'), "a")
    self.assertEqual(py_str('(1,2, 1 + 2)'), "(1, 2, 3)")
    self.assertEqual(py_str('{"a": 1 + 2}'), "{'a': 3}")

  def test_py_value(self):    
    py_value = lambda code: db.execute("select py_value(py_eval(?))", [code]).fetchone()[0]
    self.assertEqual(py_value('1 + 2'), 3)
    self.assertEqual(py_value('"a" + "b"'), "ab")
    self.assertEqual(py_value('b"yo"'), b"yo")
    self.assertEqual(py_value('.1 + .02'), 0.12000000000000001)
    self.assertEqual(py_value('None'), None)
    self.assertEqual(py_value('[1, 2, 1 + 2]'), '[1,2,3]')
    self.assertEqual(py_value('{"a": 1 + 2}'), '{"a":3}')

  def test_py_fmt(self):
    py_fmt = lambda *args: db.execute("select py_fmt({})".format(spread_args(args)), args).fetchone()[0]
    self.assertEqual(py_fmt("{}", 4), "4")
    
  def test_py_format(self):
    py_format = lambda *args: db.execute("select py_format({})".format(spread_args(args)), args).fetchone()[0]
    self.assertEqual(py_format("{}", 4), "4")
  
  def test_py_function_from_module(self):
    self.skipTest("")
  
  def test_py_functions(self):
    self.skipTest("")

  def test_py_call(self):
    py_call = lambda x: db.execute("select py_call(?)", []).fetchone()[0]
    self.assertEqual(py_call(''), None)
  
  def test_py_callable(self):
    self.skipTest("")
    py_call = lambda x: db.execute("select py_call(?)", []).fetchone()[0]
    self.assertEqual(py_call(''), None)

  def test_py_call_method(self):
    self.skipTest("")
    py_call_method = lambda x: db.execute("select py_call_method(?)", []).fetchone()[0]
    self.assertEqual(py_call_method(''), None)

  def test_py_dict(self):
    py_dict = lambda *args: db.execute("select py_value(py_dict({}))".format(spread_args(args)), args).fetchone()[0]
    self.assertEqual(py_dict('a', 1 + 2), '{"a":3}')
    #with self.assertRaisesRegex(sqlite3.OperationalError, "pattern not valid regex"):
    #  self.assertEqual(py_dict('a'), '{"a":3}')

  def test_py_list(self):
    py_list = lambda *args: db.execute("select py_value(py_list({}))".format(spread_args(args)), args).fetchone()[0]
    self.assertEqual(py_list('a', 1), '["a",1]')
  
  def test_py_tuple(self):
    py_tuple = lambda *args: db.execute("select py_value(py_tuple({}))".format(spread_args(args)), args).fetchone()[0]
    self.assertEqual(py_tuple('a', 1), '["a",1]')

  def test_py_set(self):
    py_set = lambda *args: db.execute("select py_str(py_set({}))".format(spread_args(args)), args).fetchone()[0]
    #self.assertEqual(py_set('a', 1), '{\'a\', 1}')


  def test_py_module(self):
    self.skipTest("")
    py_module = lambda x: db.execute("select py_module(?)", []).fetchone()[0]
    self.assertEqual(py_module(''), None)

  def test_py_attr(self):
    self.skipTest("")
    py_attr = lambda x: db.execute("select py_attr(?)", []).fetchone()[0]
    self.assertEqual(py_attr(''), None)

  
  def test_py_each(self):
    py_each = lambda code: execute_all("select rowid, * from py_each(py_eval(?))", [code])
    self.assertEqual(
      py_each('range(3)'),
      [
        {'rowid': 0, 'value': 0},
        {'rowid': 1, 'value': 1},
        {'rowid': 2, 'value': 2},
      ]
    )

    self.assertEqual(
      py_each('["a", "b", 1 + 2]'),
      [
        {'rowid': 0, 'value': "a"},
        {'rowid': 1, 'value': "b"},
        {'rowid': 2, 'value': 3},
      ]
    )

    self.assertEqual(
      py_each('map(lambda x: x + 1, [1, 2, 3])'),
      [
        {'rowid': 0, 'value': 2},
        {'rowid': 1, 'value': 3},
        {'rowid': 2, 'value': 4},
      ]
    )

    self.assertEqual(
      py_each('set([1, 2, 1])'),
      [
        {'rowid': 0, 'value': 1},
        {'rowid': 1, 'value': 2},
      ]
    )

    self.assertEqual(
      py_each('{"a": 1}'),
      [
        {'rowid': 0, 'value': "a"},
      ]
    )

  
class TestCoverage(unittest.TestCase):                                      
  def test_coverage(self):                                                      
    test_methods = [method for method in dir(TestPy) if method.startswith('test_')]
    funcs_with_tests = set([x.replace("test_", "") for x in test_methods])
    
    for func in FUNCTIONS:
      self.assertTrue(func in funcs_with_tests, f"{func} does not have corresponding test in {funcs_with_tests}")
    
    for module in MODULES:
      self.assertTrue(module in funcs_with_tests, f"{module} does not have corresponding test in {funcs_with_tests}")

if __name__ == '__main__':
    unittest.main()