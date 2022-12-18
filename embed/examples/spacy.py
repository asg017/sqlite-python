from sqlite_python_extensions import scalar_function, table_function, Row

import spacy
nlp = spacy.load("en_core_web_sm")


# https://spacy.io/api/doc#iter
# https://spacy.io/api/token   
@table_function(columns=["text", "pos", "dep", "tag", "prob", "ent_type", "tensor", "lemma", "norm", "shape"])
def spacy_tokens(input):
  doc = nlp(input)
  for token in doc:
    yield Row(
      text=token.text,
      pos=token.pos_,
      dep=token.dep_,
      tag=token.tag_,
      prob=token.prob,
      ent_type=token.ent_type_,
      tensor=token.tensor,
      lemma=token.lemma_,
      norm=token.norm_,
      shape=token.shape_,
    )

# https://spacy.io/api/doc#ents
# https://spacy.io/api/span
@table_function(columns=["label", "start_char", "end_char", "text"])
def spacy_entities(input):
  doc = nlp(input)
  for ent in doc.ents:
    yield Row(
      label=ent.label_,
      start_char=ent.start_char,
      end_char=ent.end_char,
      text=ent.text
    )

# https://spacy.io/api/doc#sents
# https://spacy.io/api/span
@table_function(columns=["sentence","start_char","end_char","start","end","text","tensor",])
def spacy_sentences(input):
  doc = nlp(input)
  for sent in doc.sents:
    yield Row(
      sentence=sent,
      start_char=sent.start_char,
      end_char=sent.end_char,
      start=sent.start,
      end=sent.end,
      text=sent.text,
      tensor=sent.tensor,
    )
  


@scalar_function
def spacy_doc_to_bytes(input):
  doc = nlp(input)
  return doc.to_bytes()

@scalar_function
def spacy_doc_to_json(input):
  doc = nlp(input)
  return doc.to_json()


# TODO https://spacy.io/api/doc#similarity
# TODO https://spacy.io/api/doc#to_array https://spacy.io/api/doc#from_array

# https://spacy.io/api/doc#spans
# https://spacy.io/api/doc#cats
# https://spacy.io/api/doc#noun_chunks
# https://spacy.io/api/doc#vector

pass