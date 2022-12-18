.load ../target/debug/libpy0 sqlite3_py_define
.mode box
.header on

.load ./dist/spacy0

select * from spacy_tokens("Albert's swarm was an immense concentration of the Rocky Mountain locust that swarmed the Western United States in 1875.");

select * 
from spacy_sentences("The Locust Plague of 1874, or the Grasshopper Plague of 1874, occurred when hordes of Rocky Mountain locusts invaded the Great Plains in the United States and Canada. The locust hordes covered about 2,000,000 square miles (5,200,000 km2) and caused millions of dollars' worth of damage. The swarms were so thick that they could cover the sun for up to six hours and caused millions of dollars worth of crop damage. Efforts were made to stop the infestation, including eating the locusts. Following the plague, the population of Rocky Mountain locusts continued to decline each year after 1874 and in spring 1875, many of the hatched locust eggs died due to frost, contributing to their eventual extinction.");
.exit

select typeof(spacy_doc_to_bytes("My name is alex, i'm 23 years old."));
select py_value(spacy_doc_to_json("My name is alex, i'm 23 years old."));

.exit

create table sentences as 
  select value as text
  from json_each(readfile('sentences.json')) ;

.mode list
--create table ents as
select 
  sentences.rowid as sentence,
  spacy_entities.*
from sentences
join spacy_entities(sentences.text);

--select count(*) from sentences;
--select count(*) from ents;

.exit


