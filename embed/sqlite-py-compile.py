#!/usr/bin/env python3

# Requires gcc + xxd

# TODO
# 1. Add "-I" flag for custom sqlite include directory
# 2. Respect CC env vars
# 3. If -o has no suffix, automatically add one according to OS

# Usage: `python3 compile.py my_ext.py -o my_ext.dylib --entrypoint=sqlite3_full_init`

import tempfile
import subprocess

base_c = """
#include "sqlite3ext.h"

SQLITE_EXTENSION_INIT1



int db_has_sqlite_python(sqlite3 *db) {
  sqlite3_stmt *stmt;
  int rc = sqlite3_prepare_v2(db, "select py_version()", -1, &stmt, 0);
  if (rc != SQLITE_OK) {
    return 0;
  }
  if(sqlite3_step(stmt) != SQLITE_ROW) {
    sqlite3_finalize(stmt);
    return 0;
  }
  if (sqlite3_column_type(stmt, 0) != SQLITE_NULL) {
    sqlite3_finalize(stmt);
    return 1;
  }
  return 0;
}

#ifdef _WIN32
__declspec(dllexport)
#endif
int ENTRYPOINT_NAME(sqlite3 *db, char **pzErrMsg, const sqlite3_api_routines *pApi) {
  SQLITE_EXTENSION_INIT2(pApi);

  int rc = SQLITE_OK;
  sqlite3_stmt *stmt;

  if (!db_has_sqlite_python(db)) {
    (*pzErrMsg) = sqlite3_mprintf("%s", "sqlite-python must be loaded as an extension before initiializaing TODO");
    return SQLITE_ERROR;
  }

  rc = sqlite3_prepare_v2(db, "insert into py_define(code) select ?", -1, &stmt, 0);

  if (rc != SQLITE_OK) {
    // TODO mutex for sqlite3_errmsg?
    (*pzErrMsg) = sqlite3_mprintf("%s%s", "Internal error while preparing insert py_define statement: ", sqlite3_errmsg(db));
    return rc;
  }

  rc = sqlite3_bind_text(stmt, 1, (const char *) py_code, -1, SQLITE_STATIC);
  if (rc != SQLITE_OK) {
    // TODO mutex for sqlite3_errmsg?
    (*pzErrMsg) = sqlite3_mprintf("%s%s", "Error binding text: ", sqlite3_errmsg(db));
    sqlite3_finalize(stmt);
    return SQLITE_ERROR;
  }

  if (SQLITE_DONE != sqlite3_step(stmt)) {
    // TODO mutex for sqlite3_errmsg?
    (*pzErrMsg) = sqlite3_mprintf("%s%s", "Internal error while executing py_define: ", sqlite3_errmsg(db));
    rc = SQLITE_ERROR;
  }
  sqlite3_finalize(stmt);
  
  return rc;
}

"""


# https://stackoverflow.com/questions/8924173/how-can-i-print-bold-text-in-python
class color:
   BOLD = '\033[1m'
   UNDERLINE = '\033[4m'
   END = '\033[0m'

def compile(py_source_path, output, entrypoint):
  print(f"Compiling {color.BOLD}{py_source_path}{color.END} to {color.BOLD}{output}{color.END} with entrypoint {color.BOLD}{entrypoint}{color.END}...")

  from pathlib import Path
  with tempfile.TemporaryDirectory() as tmpdir:
    f = open(Path(tmpdir) / 'tmp.c', "w")

    f.write('unsigned char py_code[] = {')

    from subprocess import Popen, PIPE, STDOUT
    p = Popen(['xxd', '-i'], stdout=PIPE, stdin=PIPE, stderr=PIPE)
    stdout_data = p.communicate(input=open(py_source_path, "rb").read())[0]
    f.write(stdout_data.decode('utf8'))
    f.write('};\n')
    f.write(base_c)
    f.close()

    subprocess.run(["gcc", 
      "-I/Users/alex/projects/sqlite-lines/sqlite",
      "-fPIC", "-shared",
      f'-DENTRYPOINT_NAME={entrypoint}',
      f.name, "-o", output
    ])
    print("✅ Compilation successful!")
    


if __name__ == "__main__":
  import argparse
  parser = argparse.ArgumentParser(prog = 'sqlite-py-compile',description = 'Compile sqlite-py scripts into loadable SQLite extensions')
  parser.add_argument('py_source')
  parser.add_argument('-o', '--output', required=True)
  parser.add_argument('--entrypoint', default="sqlite3_extension_init")

  args = parser.parse_args()
  compile(args.py_source, args.output, entrypoint=args.entrypoint)
