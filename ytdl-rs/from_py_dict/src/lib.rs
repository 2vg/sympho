use pyo3::types::{PyDict, PyList};
use pyo3::PyResult;

pub trait FromPyDict {
    fn from_py_dict(obj: &PyDict) -> PyResult<Self> where Self: Sized;
    fn from_py_dict_list(list: &PyList) -> PyResult<Vec<Self>> where Self: Sized;
}
