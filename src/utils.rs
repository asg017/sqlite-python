use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyFloat, PyInt, PyString};
use sqlite_loadable::{api, api::ValueType, prelude::*, Result};

const PYOBJECT_POINTER_NAME: &[u8] = b"pyobject0\0";

pub fn result_pyobject_pointer(context: *mut sqlite3_context, object: PyObject) {
    api::result_pointer(context, PYOBJECT_POINTER_NAME, object);
}
pub fn value_pyobject_cloned(value: &*mut sqlite3_value) -> Option<Py<PyAny>> {
    unsafe {
        if let Some(v) = api::value_pointer::<PyObject>(value, PYOBJECT_POINTER_NAME) {
            let x = (*v).clone();
            Box::into_raw(v);
            return Some(x);
        }
    }
    None
}
pub fn value_to_pyobject(py: Python, v: &*mut sqlite3_value) -> Option<PyObject> {
    match api::value_type(v) {
        ValueType::Text => api::value_text(v).ok().map(|s| PyString::new(py, s).into()),

        ValueType::Integer => Some(api::value_int(v).into_py(py)),
        ValueType::Float => Some(PyFloat::new(py, api::value_double(v)).into()),
        ValueType::Blob => {
            let b = api::value_blob(v);
            let bx = PyBytes::new_with(py, b.len(), |bytes: &mut [u8]| {
                bytes.copy_from_slice(b);
                Ok(())
            })
            .unwrap();
            Some(bx.into())
            //Some(b.into())
        }
        ValueType::Null => match value_pyobject_cloned(v) {
            Some(v) => Some(v),
            None => Some(py.None()),
        },
    }
}

pub fn result_py(context: *mut sqlite3_context, result: &PyAny) -> Result<()> {
    if let Ok(value) = result.downcast::<pyo3::types::PyString>() {
        api::result_text(context, &value.to_string_lossy())?;
    } else if let Ok(value) = result.downcast::<PyFloat>() {
      api::result_double(context, value.extract().unwrap());
  } else if let Ok(value) = result.downcast::<PyInt>() {
        api::result_int(context, value.extract().unwrap());
    } else if let Ok(value) = result.downcast::<PyBytes>() {
        api::result_blob(context, value.extract().unwrap());
    } else if result.is_none() {
        api::result_null(context);
    } else {
      result_pyobject_pointer(context, result.into())
    }
    /*else if let Ok(value) = result.downcast::<PyDict>() {
        let json = PyModule::import(py, "json").unwrap();
        let s: String = json
            .getattr("dumps")
            .unwrap()
            .call1((value,))
            .unwrap()
            .extract()
            .unwrap();
        api::result_text(context, s.as_str())?;
    } else if let Ok(value) = result.downcast::<PyList>() {
        let json = PyModule::import(py, "json").unwrap();
        let s: String = json
            .getattr("dumps")
            .unwrap()
            .call1((value,))
            .unwrap()
            .extract()
            .unwrap();
        api::result_text(context, s.as_str())?;
    }
    else {
        api::result_text(context, result.str().unwrap().to_string().as_str())?;
        //api::result_null(context);
    }*/

    Ok(())
}
