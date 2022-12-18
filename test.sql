.load target/debug/libpy0

insert into py_define(code)
  values (readfile('test.py'));

.mode box
.header on
.timer on

select pdfplumber_extract_text(5);
select pdfplumber_extract_text(page) 
from pdfplumber_pages(readfile('research/test.pdf'))
limit 4;
.exit

select pages.page_number, images.*
from pdfplumber_pages(readfile('research/test.pdf')) as pages
join pdfplumber_images(pages.page) as images
limit 10;

select * from pdfplumber_images((select page from pdfplumber_pages(readfile('research/test.pdf')) limit 1));
--select py_call_method(page, 'extract_text') from pdfplumber_pages(readfile('research/test.pdf'));
--select pdfplumber_extract_text(page) from pdfplumber_pages(readfile('research/test.pdf'));

.exit

insert into py_define(code)
  values ('
from sqlite_python_extensions import scalar_function, table_function, row, column

@scalar_function
def yo():
  return "yo"

@scalar_function
def yo2():
  return "yo2"


@scalar_function
def xadd(a, b):
  return a + b


@row
class SeriesRow:
  def __init__(self, value):
      self._value = value
  
  @column 
  def value(self):
    return self._value
  
@table_function(SeriesRow)
def xseries(start, stop):
  for x in range(start, stop):
    yield SeriesRow(x)

@table_function(SeriesRow)
def only5():
  for x in range(5):
    yield SeriesRow(x)

@row
class CharsRow:
  def __init__(self, char, i):
    self._char = char
    self._i = i
  
  @column
  def char(self):
    return self._char
  
  @column
  def i(self):
    return self._i

@table_function(CharsRow)
def chars(text):
  for i, char in enumerate(text):
    yield CharsRow(char, i)


@table_function(columns=["char", "i"])
def xchars(text):
  for i, char in enumerate(text):
    yield Row(char=char, i=i)

');

.mode box
.header on

select yo();
select yo2();

select name, narg from pragma_function_list where name in ('xadd', 'yo', 'yo2');

select xadd(1, 2);

select group_concat( printf('%s%s', name, iif(hidden, ' hidden', '')), ', ') as xseries_schema from pragma_table_xinfo('xseries');

--select * from xseries;
select * from only5;
select * from xseries(1, 10);
.schema chars

select * from chars("asdf");
.exit


select * from py_each(
  py_call(
    py_function_from_module(
      '
import time

def slow():
  for x in [1,2,3,4]:
    time.sleep(.25)
    yield x

      ',
      'slow'
    )
)
);

select * from py_each2(
  py_call(
    py_function_from_module(
      '
import time

def slow():
  for x in [1,2,3,4]:
    time.sleep(.25)
    yield x

      ',
      'slow'
    )
)
);

.timer on

select count(*) from py_each(py_eval('range(1_000_000)'));
select count(*) from py_each2(py_eval('range(1_000_000)'));
.exit

select py_version();
select py_debug();

.mode box
.header on

select 
  py_str(value)
  --py_call_method(value, '__get_item__', 0)
from py_each(
  py_call_method(
    py_attr(
      py_module('
from json import loads

def x():
  return 1

def y():
  return 1

def x():
  return 1

      '),
      '__dict__'
    ),
    'items'
  )
);

.exit


insert into py_functions(name, function)
  select 'up', py_function_from_module('
def up(s):
  return s.upper()', 'up');


insert into py_functions(name, function)
  select 'up2', py_function_from_module('
def up(s):
  return s.upper()', 'up');


select up('yo');
select up2('yo');

select * from pragma_function_list where name = 'up';

delete from py_functions where name = 'up';
select up('noooo');
select up2('yo');

select py_call(
  py_function_from_module('
def up(s):
  return s.upper()', 'up'),
  'alex'
);




.exit

select 
  py_call_method(value, '__getitem__', 0),
  py_callable(py_call_method(value, '__getitem__', 1)) as callable
from py_each(py_call_method(
  py_attr(py_module('
def x(): return 1


def my_decorator(func):
    def wrapper():
        func()
    return wrapper

@my_decorator
def z(): return 1

y = 2'), '__dict__'), 'items'));

.exit

select py_call(
  py_attr(
  py_module('
def lol():
  return "asdf"'), 'lol'
)
);

select py_call_method(
  py_module('
def lol():
  return "asdf"'), 
  'lol'
);


.exit

select py_format("asdf {}", 'a');

select py_call_method("alex garcia", "capitalize");
select py_call_method("", "zfill", 8);

.exit

select py_function('upperx', 'def up(s):
  return s.upper()', 'up');
select upperx('asdf');

select py_function('fmt', 'def fmt(base, arg1):
  return base.format(arg1)', 'fmt');
select fmt('yo: {}', 'alex');


.timer on

select count(1) from py_each(py_eval('range(1000000)'));
select count(1) from     generate_series(1, 1000000);
.exit


/*
create virtual table entities_reader using py(
  module="entities",
  function="ents",
  value(attr('label_')) as label,
  value(attr('start_char')) as start,
  value(attr('end_char')) as end,
  value(attr('text')) as text
);

select * from entities_reader('text');
*/
select 
  py_value(py_getattr(entities.value, 'label_')) as label,
  py_value(py_getattr(entities.value, 'start_char')) as start_char,
  py_value(py_getattr(entities.value, 'end_char')) as end_char,
  py_value(py_getattr(entities.value, 'text')) as text
  --sentences.value
from json_each(readfile('sentences.json')) as sentences
join py_each(
  py_call_from_module(
    py_module('
import spacy
nlp = spacy.load("en_core_web_sm")

def ents(text):
  doc = nlp(text)
  return doc.ents
'),
    'ents',
    sentences.value
  )
) as entities;

.exit

.print '-----------------------'
.mode box
.header on



select py_eval('"a" + "b"');
select py_eval('1 + 2');
select py_eval('{"A": 4}'), py_str(py_eval('{"A": 4}')), py_value(py_eval('{"A": 4}'));
select py_eval('"AABB"[:3]');
select py_eval('[i * 10 for i in range(5)]');

select 
  py_tuple('a', 2, 3.0),
  py_str(py_tuple('a', 2, 3.0)),
  py_list('a', 2, 3.0),
  py_str(py_list('a', 2, 3.0));

select py_dict(1, 2), 
  py_str(
    py_dict(
      1, 2, 
      "a", py_tuple('yo')
    )
  );

select 
  value,
  py_str(value)
from py_each(py_tuple(1, 2, 3, py_dict("a", py_list(0))));

select 
  value,
  py_str(value)
from py_each(py_eval('[x+1 for x in range(5)]'));

select 
  value,
  py_str(value)
from py_each(
  py_call(
    '
import spacy
nlp = spacy.load("en_core_web_sm")

def ents(text):
  doc = nlp(text)
  return doc.ents
', 
    'ents', 
    'My name is alex, and I work at Target.'
  )
);
.exit

/*
.timer on

select py_call(
  'def rev(s, x):
  return list(reversed(s))[:x]', 
  'rev', 
  'alex garcia',
  4
);


select py_call(
  '
def t(d):
  return type(d)', 
  't', 
  'yoyoyo'
);

select py_call(
  '
def t(d):
  return type(d)', 
  't', 
  py_tuple()--1, 2, "alex", null)
);
*/
.print '-----------------------'

.mode box
.header on
.timer off

select py_str(1), py_str(1.1), py_str('alex');
select py_str(py_tuple("a", "b", "c"));


select py_call(
  '
import usaddress
def parse(address):
  return usaddress.parse(address)',
  'parse',
  '123 Main St. Suite 100 Chicago, IL'
);
.exit

.timer on
select count(py_eval('1 + 2')) from generate_series(1, 1000);