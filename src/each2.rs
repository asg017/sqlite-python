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
pub struct PyEachTable2 {
    /// must be first
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for PyEachTable2 {
    type Aux = ();
    type Cursor = PyEachCursor2;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PyEachTable2)> {
        let vtab = PyEachTable2 {
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

    fn open(&mut self) -> Result<PyEachCursor2> {
        Ok(PyEachCursor2::new())
    }
}

#[repr(C)]
pub struct PyEachCursor2 {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    iterator: Option<PyObject>,
    value: Option<PyObject>,
}
impl PyEachCursor2 {
    fn new() -> PyEachCursor2 {
        PyEachCursor2 {
            base: unsafe { mem::zeroed() },
            rowid: 0,
            iterator: None,
            value: None,
        }
    }
}

impl VTabCursor for PyEachCursor2 {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        let value = value_pyobject_cloned(values.get(0).expect("1st required")).unwrap();

        let iterator: PyObject = Python::with_gil(|py| -> PyObject {
            let iter = PyIterator::from_object(py, &value).unwrap();
            iter.into_py(py)
        });
        self.iterator = Some(iterator);
        self.rowid = 0;
        self.next()
    }

    fn next(&mut self) -> Result<()> {
        self.rowid += 1;
        Python::with_gil(|py| {
            let iter = self.iterator.as_ref().unwrap();
            self.value = match iter.call_method0(py, "__next__") {
                Ok(value) => Some(value),
                // TODO distinguish between StopIteration and others
                Err(_) => None,
            }
            //let item = iter.next();
        });
        Ok(())
    }

    fn eof(&self) -> bool {
        self.value.is_none() //self.rowid >= self.values.as_ref().unwrap().len().try_into().unwrap()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match column(i) {
            Some(Columns::Value) => {
                //api::result_int64(context, self.value);

                //let v: &PyObject = values.get(self.rowid as usize).unwrap();
                //result_pyobject(context, self.value.to_owned());
                Python::with_gil(|py| {
                    result_py(context, self.value.as_ref().unwrap().as_ref(py)).unwrap();
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
