use std::collections::HashMap;

use derive_more::From;

use pyo3::{
    types::{IntoPyDict, PyAny, PyDict, PyList, PyModule, PyTuple}, PyErr, Python
};
use thiserror::Error;

pub type CustomOperationId = String;

#[derive(Debug)]
struct CustomOperation {
    func: pyo3::PyObject,
    input_count: u32,
}

static PYTHON_GLUE: &'static str = include_str!("pynode.py");

impl CustomOperation {
    fn from_python(obj: &PyAny) -> PyResult<Self> {
        let py = obj.py();
        let inspect = py.import("inspect")?;
        let signature = inspect.getattr("signature")?.call1((obj,))?;
        let parameter_count = signature.getattr("parameters")?.downcast::<PyList>()?.len();
        Ok(Self {
            func: obj.into(),
            input_count: parameter_count as u32,
        })
    }

    pub fn compute(&self, args: &[f64]) -> color_eyre::Result<f64> {
        let output = pyo3::Python::with_gil(|py| {
            let output = self.func.call1(py, PyTuple::new(py, args))?;
            output.extract::<f64>(py)
        })?;
        Ok(output)
    }
}

#[derive(Debug)]
pub struct CustomOperations {
    coptable: HashMap<CustomOperationId, CustomOperation>,
}

impl CustomOperations {
    fn from_python(obj: &PyDict) -> PyResult<Self> {
        Ok(Self {
            coptable: obj
                .iter()
                .map(|(k, v)| {
                    PyResult::Ok((k.extract::<String>()?, CustomOperation::from_python(v)?))
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

pub fn init() -> PyResult<'static, ()> {
    // Initializes a python3 embedded interpreter
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // Create new module
        let foo_module = PyModule::from_code(py, PYTHON_GLUE, file!(), "ode_designer")?;

        // Import and get sys.modules
        let sys = PyModule::import(py, "sys")?;
        let py_modules: &PyDict = sys.getattr("modules")?.downcast()?;

        // Insert module into sys.modules
        py_modules.set_item("ode_designer", foo_module)?;
        Ok(())
    })
}

pub type PyResult<'a, T> = Result<T, PyExtensionError<'a>>;

#[derive(Debug, From, Error)]
pub enum PyExtensionError<'a> {
    #[error(transparent)]
    PyError(PyErr),
    #[error(transparent)]
    PyDowncastError(pyo3::PyDowncastError<'a>),
    #[error("Error: {0}")]
    Other(String)
}

pub fn eval_python_code(code: &str) -> PyResult<CustomOperations> {
    Python::with_gil(|py| {
        let ode_designer_module = py.import("ode_designer")?;
        let globals = [("node", ode_designer_module.getattr("node")?)].into_py_dict(py);
        py.run(code, Some(globals), None)?;
        let node_functions = globals.get_item("__NODE_FUNCTIONS")?.ok_or_else(|| format!("Missing __NODE_FUNCTIONS"))?.downcast::<PyDict>()?;
        Ok(CustomOperations::from_python(node_functions)?)
    })
}
