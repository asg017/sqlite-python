mod constructors;
mod define;
mod each;
mod each2;
mod functions;
mod meta;
mod pyapi_table_function;
mod utils;

use pyo3::prelude::*;
use sqlite_loadable::{
    api, define_scalar_function, define_scalar_function_with_aux, define_virtual_table_writeablex,
    Error, FunctionFlags, Result,
};
use sqlite_loadable::{define_table_function, prelude::*};

use crate::{constructors::*, define::*, each::*, each2::*, functions::*, meta::*, utils::*};
use pyo3::types::PyTuple;

pub fn py_str(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(o) = value_to_pyobject(py, values.get(0).unwrap()) {
            let str = PyModule::import(py, "builtins")
                .unwrap()
                .getattr("str")
                .unwrap();
            let s: &str = str.call1((o,)).unwrap().extract().unwrap();
            api::result_text(context, s)?;
        }
        Ok(())
    })
}

pub fn py_callable(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(object) = value_to_pyobject(py, values.get(0).unwrap()) {
            let callable = PyModule::import(py, "builtins")
                .unwrap()
                .getattr("callable")
                .unwrap();
            let value: bool = callable.call1((object,)).unwrap().extract().unwrap();
            api::result_bool(context, value);
        }
        Ok(())
    })
}

pub fn py_attr(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(o) = value_to_pyobject(py, values.get(0).unwrap()) {
            let attr_name = api::value_text(values.get(1).unwrap())?;
            let o = o.getattr(py, attr_name).unwrap();
            result_py(context, o.as_ref(py))?;
        }
        Ok(())
    })
}

pub fn py_call_method(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(o) = value_to_pyobject(py, values.get(0).unwrap()) {
            let method_name = api::value_text(values.get(1).unwrap())?;
            let args = values[2..]
                .iter()
                .map(|v| match value_to_pyobject(py, v) {
                    Some(o) => Ok(o),
                    None => Err(Error::new_message("as")),
                })
                .collect::<Result<Vec<PyObject>>>()?;

            let o = o
                .call_method1(py, method_name, PyTuple::new(py, args))
                .unwrap();
            result_py(context, o.as_ref(py))?;
        }
        Ok(())
    })
}

pub fn py_value(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(result) = value_to_pyobject(py, values.get(0).unwrap()) {
            if let Ok(value) = result.extract::<i32>(py) {
                api::result_int(context, value)
            } else if let Ok(value) = result.extract::<i64>(py) {
                api::result_int64(context, value)
            } else if let Ok(value) = result.extract::<f64>(py) {
                api::result_double(context, value)
            } else if let Ok(value) = result.extract::<bool>(py) {
                api::result_bool(context, value)
            } else if let Ok(value) = result.extract::<&str>(py) {
                api::result_text(context, value)?
            } else if let Ok(value) = result.extract::<&[u8]>(py) {
                api::result_blob(context, value)
            } else if result.is_none(py) {
                api::result_null(context)
            } else {
                let json = PyModule::import(py, "json").unwrap();
                let s: String = json
                    .getattr("dumps")
                    .unwrap()
                    .call1((result,))
                    .unwrap()
                    .extract()
                    .unwrap();
                api::result_json(context, serde_json::from_str(s.as_str()).unwrap())?;
            }
        }
        Ok(())
    })
}

pub fn py_call(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        if let Some(o) = value_to_pyobject(py, values.get(0).unwrap()) {
            let args = values[1..]
                .iter()
                .map(|v| match value_to_pyobject(py, v) {
                    Some(o) => Ok(o),
                    None => Err(Error::new_message("as")),
                })
                .collect::<Result<Vec<PyObject>>>()?;

            let o = o.call1(py, PyTuple::new(py, args)).unwrap();
            result_py(context, o.as_ref(py))?;
        }
        Ok(())
    })
}

pub fn py_eval(context: *mut sqlite3_context, values: &[*mut sqlite3_value]) -> Result<()> {
    let code = api::value_text(values.get(0).unwrap())?;

    Python::with_gil(|py| -> Result<()> {
        let result = py
            .eval(code, None, None)
            .map_err(|e| Error::new_message(e.to_string().as_str()))?;
        result_py(context, result)
    })
}

pub fn py_function_from_module(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
) -> Result<()> {
    let module = api::value_text(values.get(0).unwrap())?;
    let func_name = api::value_text(values.get(1).unwrap())?;
    Python::with_gil(|py| -> Result<()> {
        let module =
            PyModule::from_code(py, module, "", "").map_err(|_| Error::new_message("asdf"))?;
        let func = module.getattr(func_name).unwrap();
        result_py(context, func)
    })?;

    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_py_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;
    define_scalar_function(db, "py_version", 0, py_version, flags)?;
    define_scalar_function(db, "py_debug", 0, py_debug, flags)?;

    define_scalar_function(db, "py_str", 1, py_str, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_callable", 1, py_callable, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_format", -1, py_format, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_fmt", -1, py_format, FunctionFlags::UTF8)?;

    define_scalar_function(db, "py_tuple", -1, py_tuple, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_list", -1, py_list, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_set", -1, py_set, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_dict", -1, py_dict, FunctionFlags::UTF8)?;

    define_scalar_function(
        db,
        "py_function_from_module",
        2,
        py_function_from_module,
        FunctionFlags::UTF8,
    )?;
    define_scalar_function(db, "py_module", 1, py_module, FunctionFlags::UTF8)?;

    define_scalar_function(db, "py_attr", 2, py_attr, FunctionFlags::UTF8)?;
    define_scalar_function(db, "py_call", -1, py_call, FunctionFlags::UTF8)?;
    define_scalar_function(
        db,
        "py_call_method",
        -1,
        py_call_method,
        FunctionFlags::UTF8,
    )?;

    define_scalar_function(db, "py_eval", 1, py_eval, FunctionFlags::UTF8)?;

    define_scalar_function(db, "py_value", 1, py_value, FunctionFlags::UTF8)?;

    define_table_function::<PyEachTable>(db, "py_each", None)?;
    define_table_function::<PyEachTable2>(db, "py_each2", None)?;
    define_virtual_table_writeablex::<PyFunctionsTable>(db, "py_functions", None)?;
    define_virtual_table_writeablex::<PyDefineTable>(db, "py_define", None)?;
    Ok(())
}
