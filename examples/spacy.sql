-- sqlite3x :memory: '.read examples/spacy.sql'
.load target/debug/libpy0

.timer on

insert into py_functions(name, function)
  select 
    'entities' as name, 
    py_function_from_module('
import spacy
nlp = spacy.load("en_core_web_sm")

def entities(text):
  doc = nlp(text)
  return doc.ents
    ', 
    'entities'
  );

create table sentences as 
  select value as text
  from json_each(readfile('examples/sentences.json')) ;

create table ents as
select 
  sentences.rowid as sentence,
  py_value(py_attr(entities.value, 'label_')) as label,
  py_value(py_attr(entities.value, 'start_char')) as start_char,
  py_value(py_attr(entities.value, 'end_char')) as end_char,
  py_value(py_attr(entities.value, 'text')) as text
from sentences
join py_each(entities(sentences.text)) as entities;
select count(*) from sentences;
select count(*) from ents;
.exit

