use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyTuple};
use pyo3::PyObject;
use sqlite_loadable::prelude::*;
use sqlite_loadable::{
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

use std::{mem, os::raw::c_int};

#[derive(Clone)]
pub struct AuxColumn {
    pub name: String,
    pub hidden: bool,
    pub required: bool,
}
pub struct Aux {
    pub(crate) sql: String,
    pub columns: Vec<AuxColumn>,
    pub generator: PyObject,
}
#[repr(C)]
pub struct PyApiTableBuilder {
    /// must be first
    base: sqlite3_vtab,
    db: *mut sqlite3,
    rowid: i64,
    generator: PyObject,
    columns: Vec<AuxColumn>,
}

impl<'vtab> VTab<'vtab> for PyApiTableBuilder {
    type Aux = Aux;
    type Cursor = PyApiTableCursor<'vtab>;

    fn connect(
        db: *mut sqlite3,
        aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, Self)> {
        let aux = aux.unwrap().to_owned();
        let vtab = PyApiTableBuilder {
            base: unsafe { mem::zeroed() },
            db,
            rowid: 0,
            generator: aux.generator.clone(),
            columns: aux.columns.clone(),
        };
        // TODO db.config(VTabConfig::Innocuous)?;
        Ok((aux.sql.clone(), vtab))
    }
    fn destroy(&self) -> Result<()> {
        Ok(())
    }

    fn best_index(&self, mut info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        let mut required_column_constraints = HashSet::new();
        let mut argv = 1;
        // Assert all hidden required columns have a EQ constraint
        for mut constraint in info.constraints() {
            let column_idx = constraint.column_idx() as usize;
            match self.columns.get(column_idx) {
                Some(column) => {
                    if constraint.usable() && column.required {
                        required_column_constraints.insert(column_idx);
                        constraint.set_omit(true);
                        constraint.set_argv_index(argv);
                        argv += 1;
                    }
                }
                None => todo!(),
            }
        }
        for (i, column) in self.columns.iter().enumerate() {
            if column.required && !required_column_constraints.contains(&i) {
                return Err(BestIndexError::Constraint);
            }
        }
        info.set_estimated_cost(100000.0);
        info.set_estimated_rows(100000);
        info.set_idxnum(1);
        Ok(())
    }

    fn open(&mut self) -> Result<PyApiTableCursor> {
        Ok(PyApiTableCursor::new(self))
    }
}
use std::collections::HashSet;

use crate::utils::{result_pyobject_as_value, value_to_pyobject};

#[repr(C)]
pub struct PyApiTableCursor<'a> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    generator: &'a PyObject,
    columns: Vec<AuxColumn>,
    iterator: Option<PyObject>,
    value: Option<PyObject>,
}
impl<'a> PyApiTableCursor<'a> {
    fn new(table: &'a PyApiTableBuilder) -> PyApiTableCursor<'a> {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PyApiTableCursor {
            base,
            rowid: 0,
            generator: &table.generator,
            columns: table.columns.clone(),
            iterator: None,
            value: None,
        }
    }
}

impl<'a> VTabCursor for PyApiTableCursor<'a> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        // call the generate function with proper arguments
        //let pattern = values.get(0).unwrap().text()?;
        //let contents = values.get(1).unwrap().text()?;
        let iterator: PyObject = Python::with_gil(|py| -> PyObject {
            let elements: Vec<PyObject> = values
                .iter()
                .filter_map(|v| value_to_pyobject(py, v))
                .collect();
            let iter = PyIterator::from_object(
                py,
                &self
                    .generator
                    .call1(py, PyTuple::new(py, elements))
                    .unwrap(),
            )
            .unwrap();
            iter.into_py(py)
        });
        self.iterator = Some(iterator);
        self.rowid = 0;
        self.next()
    }

    fn next(&mut self) -> Result<()> {
        Python::with_gil(|py| {
            let iter = self.iterator.as_ref().unwrap();
            self.value = match iter.call_method0(py, "__next__") {
                Ok(value) => Some(value),
                Err(err) => {
                    if err.is_instance_of::<PyStopIteration>(py) {
                        None
                    } else {
                        todo!("iteration error: {}", err)
                    }
                }
            }
            //let item = iter.next();
        });
        Ok(())
    }

    fn eof(&self) -> bool {
        self.value.is_none()
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let column = self.columns.get(i as usize).unwrap();
        Python::with_gil(|py| {
            let v = self
                .value
                .as_ref()
                .unwrap()
                .getattr(py, column.name.as_str())
                .unwrap();
            //.call_method0(py, column.name.as_str())
            //.unwrap();
            result_pyobject_as_value(py, context, v.as_ref(py)).unwrap();
        });
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
