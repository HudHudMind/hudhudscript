//! PYTHON0002: Value16 ↔ Python type conversion.

use hudhudscript_bytecode::Value16;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyBool, PyFloat, PyString};

pub fn py_to_value(obj: &Bound<'_, PyAny>) -> PyResult<Value16> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut map = hudhudscript_bytecode::ObjMap::default();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            let val = py_to_value(&v)?;
            map.insert(key, val);
        }
        Ok(Value16::object(map))
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let items: Vec<Value16> = list.iter().map(|v| py_to_value(&v)).collect::<PyResult<_>>()?;
        Ok(Value16::array(items))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(Value16::bool_(b))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(Value16::number(f))
    } else if let Ok(s) = obj.extract::<String>() {
        Ok(Value16::string(s))
    } else if obj.is_none() {
        Ok(Value16::null())
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(Value16::int(i))
    } else {
        let s: String = obj.str()?.to_string();
        Ok(Value16::string(s))
    }
}

pub fn value_to_py(py: Python<'_>, val: &Value16) -> PyResult<PyObject> {
    if let Some(b) = val.as_bool() {
        Ok(b.to_object(py))
    } else if let Some(n) = val.as_number() {
        Ok(n.to_object(py))
    } else if let Some(i) = val.as_int() {
        Ok(i.to_object(py))
    } else if let Some(s) = val.as_string() {
        Ok(s.to_object(py))
    } else if let Some(arr) = val.as_array() {
        let list = PyList::empty(py);
        for v in arr.iter() {
            list.append(value_to_py(py, v)?)?;
        }
        Ok(list.into())
    } else if let Some(obj) = val.as_object() {
        let dict = PyDict::new(py);
        for (k, v) in obj.iter() {
            let key_str = hudhudscript_bytecode::interner::resolve(hudhudscript_bytecode::interner::SymbolId(k.0));
            dict.set_item(key_str, value_to_py(py, v)?)?;
        }
        Ok(dict.into())
    } else if val.is_null() {
        Ok(py.None())
    } else {
        Ok(py.None())
    }
}
