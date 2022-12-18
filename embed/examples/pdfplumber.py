from sqlite_python_extensions import scalar_function, table_function, Row

import pdfplumber
from io import BytesIO

@scalar_function
def pdfplumber_version():
  return "v0.0.0"

@scalar_function
def pdfplumber_debug():
  return f"""py_pdfplumber: v0.0.0
pdfplumber: {pdfplumber.__version__}"""


@table_function(columns=["page", "width", "height", "page_number", "number_images"])
def pdfplumber_pages(pdf_data):
  with pdfplumber.open(BytesIO(pdf_data)) as pdf:
    for page in pdf.pages:
      yield Row(
        page=page, 
        width=page.width, 
        height=page.height, 
        page_number=page.page_number,
        number_images=len(page.images)
      )

@table_function(columns=["x0", "y0", "x1", "y1", "width", "height"])
def pdfplumber_images(page, ):
  for image in page.images:
    yield Row(
      x0=image.get('x0'),
      y0=image.get('y0'),
      x1=image.get('x1'),
      y1=image.get('y1'),
      width=image.get('width'),
      height=image.get('height'),
    )

@scalar_function
def pdfplumber_extract_text(page):
  if not isinstance(page, pdfplumber.page.Page):
    raise Exception("input to pdfplumber_extract_text should be a pdfplumber Page, instead found", type(page))
  return page.extract_text()
