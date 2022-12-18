# sqlite-python documentation

To start, clone this repo, run `cargo build`, and see if it works with the `sqlite3` CLI:

```
cargo build
sqlite3 :memory: '.load ./target/debug/libpy0' 'select py_version()'
```

To try the "write SQLite extensions in Python" feature, create an `extension.py` file with the following contents:

```py
# extension.py
from sqlite_python_extensions import scalar_function, table_function, Row

@scalar_function
def hello(name):
  return f"Hello, {name}!"

@table_function(columns=["value", "progress"])
def series(start, stop):
  for value in range(start, stop+1):
    yield Row(value=value, progress=(value - start) / (stop-start))
```

```sql

.load ./target/debug/libpy0
insert into py_define(code) values (readfile('extension.py'));

select hello('world');
select * from series(1, 8);
```
