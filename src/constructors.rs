use crate::utils::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple, PyString, PySet};
use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, Error, Result};

pub fn py_module(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let code = api::value_text(values.get(0).unwrap())?;
    Python::with_gil(|py| -> Result<()> {
        let module =
            PyModule::from_code(py, code, "", "").map_err(|_| Error::new_message("asdf"))?;
            result_pyobject_pointer(context, module.into());
        Ok(())
    })
}

pub fn py_format(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
  let base = api::value_text(values.get(0).unwrap())?;
  Python::with_gil(|py| -> Result<()> {
      let base = PyString::new(py, base);
      
      let args: Result<Vec<PyObject>> = values[1..]
          .iter()
          .map(|v| match value_to_pyobject(py, v) {
              Some(o) => Ok(o),
              None => Err(Error::new_message("as")),
          })
          .collect();
      let args = PyTuple::new(py, args?);
      let result = base.getattr("format").unwrap().call1(args).unwrap();
      result_py(context, result)?;

      Ok(())
  })
}

pub fn py_tuple(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
  Python::with_gil(|py| -> Result<()> {
      let elements: Result<Vec<PyObject>> = values
          .iter()
          .map(|v| match value_to_pyobject(py, v) {
              Some(o) => Ok(o),
              None => Err(Error::new_message("as")),
          })
          .collect();

      let tuple = PyTuple::new(py, elements?);
      result_pyobject_pointer(context, tuple.into());

      Ok(())
  })
}

pub fn py_list(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
  Python::with_gil(|py| -> Result<()> {
      let elements: Result<Vec<PyObject>> = values
          .iter()
          .map(|v| match value_to_pyobject(py, v) {
              Some(o) => Ok(o),
              None => Err(Error::new_message("as")),
          })
          .collect();

      let tuple = PyList::new(py, elements?);
      result_pyobject_pointer(context, tuple.into());

      Ok(())
  })
}

pub fn py_set(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
  Python::with_gil(|py| -> Result<()> {
      let elements: Result<Vec<PyObject>> = values
          .iter()
          .map(|v| match value_to_pyobject(py, v) {
              Some(o) => Ok(o),
              None => Err(Error::new_message("as")),
          })
          .collect();

      let set = PySet::new(py, &elements?).unwrap();
      result_pyobject_pointer(context, set.into());

      Ok(())
  })
}

pub fn py_dict(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        let elements: Result<Vec<PyObject>> = values
            .iter()
            .map(|v| match value_to_pyobject(py, v) {
                Some(o) => Ok(o),
                None => Err(Error::new_message("as")),
            })
            .collect();
        let sequence = PyList::empty(py);
        for pair in elements?.chunks(2) {
            let key = pair.get(0).unwrap();
            let value = pair.get(1).unwrap();
            let item = PyTuple::new(py, [key, value]);
            sequence.append(item).unwrap();
        }
        //let tuple = PyDict::new(py
        let dict = PyDict::from_sequence(py, sequence.into()).unwrap();
        result_pyobject_pointer(context, dict.into());

        Ok(())
    })
}
