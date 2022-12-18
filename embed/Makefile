dist/pysimple0.dylib: examples/simple.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_pysimple_init

dist/py_pdfplumber0.dylib: examples/pdfplumber.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_pypdfplumber_init

dist/usaddress0.dylib: examples/usaddress.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_usaddress_init

dist/spacy0.dylib: examples/spacy.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_spacy_init

dist/a.dylib: examples/a.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_a_init

dist/b.dylib: examples/b.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_b_init

dist/scalar.dylib: examples/scalar.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_scalar_init

dist/params.dylib: examples/params.py
	./sqlite-py-compile.py $< -o $@ --entrypoint sqlite3_params_init

simple: dist/pysimple0.dylib 
pdfplumber: dist/py_pdfplumber0.dylib
a: dist/a.dylib
b: dist/b.dylib
scalar: dist/scalar.dylib
usaddress: dist/usaddress0.dylib
spacy: dist/spacy0.dylib
params: dist/params.dylib

all: simple pdfplumber a b scalar usaddress spacy params

.PHONY: all simple pdfplumber a b scalar usaddress spacy params