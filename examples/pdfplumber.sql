-- sqlite3x examples/rioters.db '.read examples/pdfplumber.sql'

.load target/debug/libpy0
--select contents from case_documents;



insert into py_functions(name, function)
  select 
    'pdf_pages' as name, 
    py_function_from_module('
import io

import pdfplumber

def pdf_pages(input_pdf):
  with pdfplumber.open(io.BytesIO(input_pdf)) as pdf:
    for page in pdf.pages:
      yield page
    ', 
    'pdf_pages'
  );

.mode box
.header on

select 
  py_attr(value, 'page_number') as page_number,
  py_attr(value, 'width') as width,
  py_attr(value, 'height') as height,
  py_call_method(value, 'extract_text') as text
from case_documents
join py_each(pdf_pages(contents))
limit 10;


/*

*/