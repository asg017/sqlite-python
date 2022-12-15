
import io
import pdfplumber

@table_function(columns=[attrs('page_number', 'width', 'height')])

class Page(Row):
  @column
  def page_number():
    self.page_number
  
def pdf_pages(input_pdf):
  with pdfplumber.open(io.BytesIO(input_pdf)) as pdf:
    for page in pdf.pages:
      yield page
    