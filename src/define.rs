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
use sqlite_loadable::{define_scalar_function_with_aux, define_table_function, prelude::*};

use crate::pyapi_table_function::{Aux, AuxColumn, PyApiTableBuilder};
use crate::utils::*;

use std::collections::HashMap;
use std::ops::Index;
use std::{mem, os::raw::c_int};

use crate::utils::value_pyobject_cloned;

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
    defined: HashMap<i64, (String, PyObject)>,
}

use sqlite_loadable::table::UpdateOperation;
impl<'vtab> VTab<'vtab> for PyDefineTable {
    type Aux = ();
    type Cursor = PyDefineCursor<'vtab>;

    fn connect(
        db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, PyDefineTable)> {
        let base: sqlite3_vtab = unsafe { mem::zeroed() };
        let vtab = PyDefineTable {
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

    fn open(&mut self) -> Result<PyDefineCursor> {
        Ok(PyDefineCursor::new(&self.defined))
    }
}

static API_PY_CODE: &str = include_str!("api.py");
use pyo3::iter;

impl<'vtab> VTabWriteable<'vtab> for PyDefineTable {
    fn update(&'vtab mut self, operation: UpdateOperation, p_rowid: *mut i64) -> Result<()> {
        match operation {
            UpdateOperation::Insert { values, rowid } => {
                println!("rowid={:?}, len={}", rowid, values.len());
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
                    let api_module = PyModule::from_code(
                        py,
                        API_PY_CODE,
                        "sqlite_python_extensions",
                        "sqlite_python_extensions",
                    )
                    .unwrap();
                    let scalar_function_class = api_module.getattr("ScalarFunction").unwrap();

                    let user_module = PyModule::from_code(py, code, "", "").unwrap();
                    let items = user_module
                        .getattr("__dict__")
                        .unwrap()
                        .call_method0("items")
                        .unwrap()
                        .call_method0("__iter__")
                        .unwrap();
                    //println!("items={:?}", items);

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
                    while let Ok(item) = items.call_method0("__next__") {
                        if item
                            .get_item(1)
                            .unwrap()
                            .hasattr("scalar_function")
                            .unwrap()
                        {
                            let name = item.get_item(0).unwrap().str().unwrap().to_str().unwrap();
                            let func = item.get_item(1).unwrap().getattr("function").unwrap();
                            let argc = item
                                .get_item(1)
                                .unwrap()
                                .getattr("argc")
                                .unwrap()
                                .extract()
                                .unwrap();

                            define_scalar_function_with_aux(
                                self.db,
                                name,
                                argc,
                                def,
                                FunctionFlags::UTF8,
                                func.into(),
                            )
                            .unwrap();
                        } else if item.get_item(1).unwrap().hasattr("table_function").unwrap() {
                            println!("defining table func");
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

                    //for x in items {
                    //    //println!("{:?}", x);
                    //    if x.hasattr("scalar_function").unwrap() {
                    //        println!("sclaar")
                    //    }
                    //}
                });
            }
            _ => todo!(),
        };
        Ok(())
    }
}

#[repr(C)]
pub struct PyDefineCursor<'a> {
    /// Base class. Must be first
    base: sqlite3_vtab_cursor,
    rowid: i64,
    defined: &'a HashMap<i64, (String, PyObject)>,
}
impl PyDefineCursor<'_> {
    fn new(defined: &HashMap<i64, (String, PyObject)>) -> PyDefineCursor {
        let base: sqlite3_vtab_cursor = unsafe { mem::zeroed() };
        PyDefineCursor {
            base,
            rowid: 0,
            defined,
        }
    }
}

impl VTabCursor for PyDefineCursor<'_> {
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
            _ => todo!(),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.rowid)
    }
}
