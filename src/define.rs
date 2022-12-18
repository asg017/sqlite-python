//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::PyObject;
use sqlite_loadable::table::VTabWriteable;
use sqlite_loadable::{
    api,
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};
use sqlite_loadable::{define_scalar_function_with_aux, define_table_function, prelude::*, Error};

use crate::pyapi_table_function::{Aux, AuxColumn, PyApiTableBuilder};
use crate::utils::*;

use std::{mem, os::raw::c_int};

fn scalar_defintion(
    context: *mut sqlite3_context,
    values: &[*mut sqlite3_value],
    func: &PyObject,
) -> Result<()> {
    Python::with_gil(|py| -> Result<()> {
        let elements: Vec<PyObject> = values
            .iter()
            .filter_map(|v| value_to_pyobject(py, v))
            .collect();
        match func.call1(py, PyTuple::new(py, elements)) {
            Ok(result) => result_py(context, result.as_ref(py)),
            Err(err) => return Err(Error::new_message(err.to_string().as_str())),
        }
    })
}

static CREATE_SQL: &str = "CREATE TABLE x(code hidden)";
enum Columns {
    Code,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Code),
        _ => None,
    }
}

#[repr(C)]
pub struct PyDefineTable {
    /// must be first
    base: sqlite3_vtab,
    db: *mut sqlite3,
    rowid: i64,
}

use sqlite_loadable::table::UpdateOperation;
impl<'vtab> VTab<'vtab> for PyDefineTable {
    type Aux = ();
    type Cursor = PyDefineCursor;

    fn connect(
        db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PyDefineTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PyDefineTable { base, db, rowid: 0 };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);
        Ok(())
    }

    fn open(&mut self) -> Result<PyDefineCursor> {
        Ok(PyDefineCursor::new())
    }
}

static API_PY_CODE: &str = include_str!("api.py");

impl<'vtab> VTabWriteable<'vtab> for PyDefineTable {
    fn update(&'vtab mut self, operation: UpdateOperation, _p_rowid: *mut i64) -> Result<()> {
        match operation {
            UpdateOperation::Insert { values, rowid: _ } => {
                self.rowid += 1;
                let mut code = None;
                for (idx, value) in values.iter().enumerate() {
                    match column(idx.try_into().unwrap()) {
                        Some(Columns::Code) => {
                            code = Some(api::value_text(value)?);
                        }
                        None => todo!(),
                    }
                }
                let code = code.unwrap();
                Python::with_gil(|py| {
                    // TODO elsewhere?
                    let _api_module = PyModule::from_code(
                        py,
                        API_PY_CODE,
                        "sqlite_python_extensions",
                        "sqlite_python_extensions",
                    )
                    .unwrap();
                    let name = format!("m{}", self.rowid);
                    let user_module =
                        PyModule::from_code(py, code, name.as_str(), name.as_str()).unwrap();
                    let items = user_module
                        .getattr("__dict__")
                        .unwrap()
                        .call_method0("items")
                        .unwrap()
                        .call_method0("__iter__")
                        .unwrap();

                    while let Ok(item) = items.call_method0("__next__") {
                        if item
                            .get_item(1)
                            .unwrap()
                            .hasattr("scalar_function")
                            .unwrap()
                        {
                            let name = item.get_item(0).unwrap().str().unwrap().to_str().unwrap();
                            let scalar = item.get_item(1).unwrap();
                            let func = scalar.getattr("function").unwrap();
                            //let argc = scalar.getattr("argc").unwrap().extract().unwrap();
                            let argc_required: i32 =
                                scalar.getattr("argc_required").unwrap().extract().unwrap();
                            let argc_optional: i32 =
                                scalar.getattr("argc_optional").unwrap().extract().unwrap();

                            for i in 0..argc_optional + 1 {
                                define_scalar_function_with_aux(
                                    self.db,
                                    name,
                                    argc_required + i,
                                    scalar_defintion,
                                    FunctionFlags::UTF8,
                                    func.into(),
                                )
                                .unwrap();
                            }
                        } else if item.get_item(1).unwrap().hasattr("table_function").unwrap() {
                            let tf = item.get_item(1).unwrap();
                            let name = tf.getattr("name").unwrap().extract().unwrap();
                            let sql = tf.getattr("sql").unwrap().extract().unwrap();
                            let generator = tf.getattr("generator").unwrap().into_py(py);
                            let columns = tf
                                .getattr("columns")
                                .unwrap()
                                .iter()
                                .unwrap()
                                .map(|c| {
                                    let c = c.unwrap();
                                    let name = c.getattr("name").unwrap().extract().unwrap();
                                    let hidden = c.getattr("hidden").unwrap().extract().unwrap();
                                    let required =
                                        c.getattr("required").unwrap().extract().unwrap();

                                    AuxColumn {
                                        name,
                                        hidden,
                                        required,
                                    }
                                })
                                .collect();
                            define_table_function::<PyApiTableBuilder>(
                                self.db,
                                name,
                                Some(Aux {
                                    sql,
                                    columns,
                                    generator,
                                }),
                            )
                            .unwrap();
                        }
                    }
                });
            }
            _ => todo!(),
        };
        Ok(())
    }
}

#[repr(C)]
pub struct PyDefineCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
}
impl PyDefineCursor {
    fn new() -> PyDefineCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PyDefineCursor { base, rowid: 0 }
    }
}

impl VTabCursor for PyDefineCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        //let pattern = values.get(0).unwrap().text()?;
        //let contents = values.get(1).unwrap().text()?;
        self.rowid = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        true
    }

    fn column(&self, _context: *mut sqlite3_context, _i: c_int) -> Result<()> {
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
