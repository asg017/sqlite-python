//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyTuple};
use pyo3::PyObject;
use sqlite_loadable::scalar::delete_scalar_function;
use sqlite_loadable::table::{VTabWriteable, VTabWriteableWithTransactions};
use sqlite_loadable::{
    api,
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};
use sqlite_loadable::{define_scalar_function_with_aux, prelude::*};

use crate::utils::*;

use std::collections::HashMap;
use std::ops::Index;
use std::{mem, os::raw::c_int};

use crate::utils::value_pyobject_cloned;

static CREATE_SQL: &str = "CREATE TABLE x(name text, function hidden)";
enum Columns {
    Name,
    Function,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Name),
        1 => Some(Columns::Function),
        _ => None,
    }
}

#[repr(C)]
pub struct PyFunctionsTable {
    /// must be first
    base: sqlite3_vtab,
    db: *mut sqlite3,
    rowid: i64,
    defined: HashMap<i64, (String, PyObject)>,
}

use sqlite_loadable::table::UpdateOperation;
impl<'vtab> VTab<'vtab> for PyFunctionsTable {
    type Aux = ();
    type Cursor = PyFunctionsCursor<'vtab>;

    fn connect(
        db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PyFunctionsTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PyFunctionsTable {
            base,
            db,
            rowid: 0,
            defined: HashMap::new(),
        };
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

    fn open(&mut self) -> Result<PyFunctionsCursor> {
        Ok(PyFunctionsCursor::new(&self.defined))
    }
}

impl<'vtab> VTabWriteable<'vtab> for PyFunctionsTable {
    fn update(&'vtab mut self, operation: UpdateOperation, p_rowid: *mut i64) -> Result<()> {
        match operation {
            UpdateOperation::Insert { values, rowid } => {
                println!("rowid={:?}, len={}", rowid, values.len());
                if let Some(rowid) = rowid {
                    println!("{}", api::value_int64(rowid));
                }
                let mut name = None;
                let mut func = None;
                for (idx, value) in values.iter().enumerate() {
                    match column(idx.try_into().unwrap()) {
                        Some(Columns::Name) => {
                            name = Some(api::value_text(value)?);
                        }
                        Some(Columns::Function) => {
                            func = value_pyobject_cloned(value);
                        }
                        None => todo!(),
                    }
                }

                let name = name.expect("name");
                let func = func.expect("func");
                let argc = Python::with_gil(|py| -> i32 {
                    func.getattr(py, "__code__")
                        .unwrap()
                        .getattr(py, "co_argcount")
                        .unwrap()
                        .extract(py)
                        .unwrap()
                });
                fn def(
                    context: *mut sqlite3_context,
                    values: &[*mut sqlite3_value],
                    func: &PyObject,
                ) -> Result<()> {
                    Python::with_gil(|py| -> Result<()> {
                        let elements: Vec<PyObject> = values
                            .iter()
                            .filter_map(|v| value_to_pyobject(py, v))
                            .collect();
                        let result = func.call1(py, PyTuple::new(py, elements)).unwrap();
                        result_py(context, result.as_ref(py))?;
                        Ok(())
                    })?;
                    //api::result_text(context, code)?;
                    Ok(())
                }
                self.defined
                    .insert(self.rowid, (name.to_string(), func.to_owned()));
                self.rowid += 1;
                define_scalar_function_with_aux(
                    self.db,
                    name,
                    argc,
                    def,
                    FunctionFlags::UTF8,
                    func,
                )?;

                //delete_scalar_function(self.db, "up", 1, FunctionFlags::UTF8)?;
            }
            // TODO would sqlite3_unlock_notify fix this
            UpdateOperation::Delete(value) => {
                let rowid = api::value_int64(value);
                println!("deleting {rowid}");
                let (name, function) = self.defined.get(&rowid).unwrap();
                delete_scalar_function(self.db, name, 1, FunctionFlags::UTF8)?;
                self.defined.remove(&rowid);
            }
            _ => todo!(),
            //UpdateOperation::In
        };
        Ok(())
    }
}

#[repr(C)]
pub struct PyFunctionsCursor<'a> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    defined: &'a HashMap<i64, (String, PyObject)>,
}
impl PyFunctionsCursor<'_> {
    fn new(defined: &HashMap<i64, (String, PyObject)>) -> PyFunctionsCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PyFunctionsCursor {
            base,
            rowid: 0,
            defined,
        }
    }
}

impl VTabCursor for PyFunctionsCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
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
        self.rowid >= self.defined.len().try_into().unwrap()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let (name, function) = self.defined.get(&self.rowid).unwrap();
        match column(i) {
            Some(Columns::Name) => {
                api::result_text(context, name);
            }
            Some(Columns::Function) => {
                //api::result_null(context);
                Python::with_gil(|py| {
                    result_py(context, function.as_ref(py));
                });
            }
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
