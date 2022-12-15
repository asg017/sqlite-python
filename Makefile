example-pdfplumber:
	sqlite3x examples/rioters.db '.read examples/pdfplumber.sql'

example-spacy:
	sqlite3x :memory: '.read examples/spacy.sql'

example-usaddress:
	sqlite3x :memory: '.read examples/usaddress.sql'

example-ffmpeg:
	sqlite3x :memory: '.read examples/ffmpeg.sql'

examples: example-pdfplumber example-spacy example-usaddress example-ffmpeg

.PHONY: example-pdfplumber example-spacy example-usaddress example-ffmpeg