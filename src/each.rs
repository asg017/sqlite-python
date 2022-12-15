//! cargo build --example series
//! sqlite3 :memory: '.read examples/test.sql'

use pyo3::prelude::*;
use pyo3::types::PyIterator;
use pyo3::PyObject;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    table::{BestIndexError, ConstraintOperator, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use crate::utils::*;

use std::{mem, os::raw::c_int};

use crate::utils::value_pyobject_cloned;

static CREATE_SQL: &str = "CREATE TABLE x(value, object hidden)";
enum Columns {
    Value,
    Object,
}
fn column(index: i32) -> Option<Columns> {
    match index {
        0 => Some(Columns::Value),
        1 => Some(Columns::Object),
        _ => None,
    }
}

#[repr(C)]
pub struct PyEachTable {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for PyEachTable {
    type Aux = ();
    type Cursor = PyEachCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PyEachTable)> {
        let vtab = PyEachTable {
            base: unsafe { mem::zeroed() },
        };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((CREATE_SQL.to_owned(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut has_object = false;
        for mut constraint in info.constraints() {
            match column(constraint.column_idx()) {
                Some(Columns::Object) => {
                    if constraint.usable() && constraint.op() == Some(ConstraintOperator::EQ) {
                        constraint.set_omit(true);
                        constraint.set_argv_index(1);
                        has_object = true;
                    } else {
                        return Err(BestIndexError::Constraint);
                    }
                }
                _ => todo!(),
            }
        }
        if !has_object {
            return Err(BestIndexError::Error);
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);

        Ok(())
    }

    fn open(&mut self) -> Result<PyEachCursor> {
        Ok(PyEachCursor::new())
    }
}

#[repr(C)]
pub struct PyEachCursor {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    values: Option<Vec<PyObject>>,
}
impl PyEachCursor {
    fn new() -> PyEachCursor {
        PyEachCursor {
            base: unsafe { mem::zeroed() },
            rowid: 0,
            values: None,
        }
    }
}

impl VTabCursor for PyEachCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let value = value_pyobject_cloned(values.get(0).expect("1st required")).unwrap();

        let values: Vec<PyObject> = Python::with_gil(|py| -> Vec<PyObject> {
            let iter = PyIterator::from_object(py, &value).unwrap();

            Vec::from_iter(iter.map(|x| x.unwrap().into_py(py)))
        });
        self.values = Some(values);
        self.rowid = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.rowid >= self.values.as_ref().unwrap().len().try_into().unwrap()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Value) => {
                //api::result_int64(context, self.value);
                let values = self.values.as_ref().unwrap();
                let v: &PyObject = values.get(self.rowid as usize).unwrap();
                //result_pyobject(context, v.to_owned());
                Python::with_gil(|py| {
                    result_py(context, v.as_ref(py)).unwrap();
                })
                //result_py(context, v.to_owned());
            }
            Some(Columns::Object) => {
                todo!();
            }
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
