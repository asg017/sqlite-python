use pyo3::prelude::*;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{api, Result};

pub fn py_version(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    api::result_text(context, &format!("v{}", env!("CARGO_PKG_VERSION")))?;
    Ok(())
}

pub fn py_debug(context: *mut sqlite3_context, _values: &[*mut sqlite3_value]) -> Result<()> {
    let version = Python::with_gil(|py| -> Result<String> { Ok(py.version().to_owned()) }).unwrap();

    api::result_text(
        context,
        &format!(
            "Version: v{}
Source: {}
Python version: {}",
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH"),
            version,
        ),
    )?;
    Ok(())
}
