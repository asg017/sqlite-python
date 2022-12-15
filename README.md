```sql

insert into python_functions(name, code)
  select 'reversed', '', 'attr';


select py_function_from_module('module str or obj', 'funcname');
select name, value from py_module_functions('module str or obj');
```

## TODO

- [ ] write sqlite extensions in python

```python
# myext.py
import scalar, table_function from sqlite_python_loadable

@scalar
def pyx_add(a, b):
  return a + b

# config for best_index, orderby, projection pushdown, predicate pushdowns?
@table_function(['value'])
def pyx_series(min, max, config=None):
  for x in range(min, max)
    yield x
```

```
sqlite_python_loadable generate myext.py -o myext.c
gcc -fPIC -shared myext.c -o myext0.dylib
```

```sql
.load py0
.load myext0

select pyx_add(1, 2);

select * from pyx_series(1, 100);
```

## Examples

- [ ] usaddress
- [ ] playwright
- [ ] pdfplumber
- [ ] ffmpeg
- [ ] spacy
- [ ] whisper
- [ ] tensorflow
