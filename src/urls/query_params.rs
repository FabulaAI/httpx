use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    vec::IntoIter,
};

use indexmap::IndexMap;
use pyo3::{
    exceptions::{PyAssertionError, PyKeyError, PyRuntimeError},
    prelude::*,
    types::{PyBool, PyDict, PyList, PyTuple},
    IntoPyObjectExt,
};

fn primitive_value_to_str(value: &Bound<'_, PyAny>) -> PyResult<String> {
    if value.is_instance_of::<PyBool>() {
        let bool_value = value.extract::<bool>()?;
        Ok(bool_value.to_string())
    } else if value.is_none() {
        Ok("".to_string())
    } else {
        Ok(value.to_string())
    }
}

fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            b' ' => "+".to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

#[pyclass(eq, frozen, str, hash)]
#[derive(Debug, Clone)]
pub struct QueryParams {
    params: IndexMap<String, Vec<String>>,
}

#[pymethods]
impl QueryParams {
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    pub fn new(args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        if args.len() > 1 {
            return Err(PyAssertionError::new_err("Too many arguments."));
        }

        match args.get_item(0) {
            Ok(item) => QueryParams::from_pyany(&item),
            Err(_) => match kwargs {
                Some(kwargs) => QueryParams::from_pydict(kwargs),
                None => {
                    return Ok(QueryParams {
                        params: IndexMap::new(),
                    })
                }
            },
        }
    }

    pub fn keys(&self) -> Vec<String> {
        self.params.keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<String> {
        let mut values = Vec::with_capacity(self.params.len());
        for values_list in self.params.values() {
            if let Some(value) = values_list.first() {
                values.push(value.clone());
            }
        }
        values
    }

    pub fn items(&self) -> Vec<(String, String)> {
        let mut items = Vec::with_capacity(self.params.len());
        for (key, values) in &self.params {
            if !values.is_empty() {
                items.push((key.clone(), values[0].clone()));
            }
        }
        items
    }

    /// # NOTE
    /// Think about the performance of this method.
    ///
    /// Iterating over all items and creating clones of keys and values may be not efficent.
    /// But if we return references like `Vec<(&String, &String)>` it can lead to lifetime issues.
    pub fn multi_items(&self) -> Vec<(String, String)> {
        let mut items = Vec::new();
        for (key, values) in &self.params {
            for value in values {
                items.push((key.clone(), value.clone()));
            }
        }
        items
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get(&self, py: Python<'_>, key: String, default: Option<Bound<'_, PyAny>>) -> PyResult<Option<Py<PyAny>>> {
        match self.params.get(&key) {
            Some(values) => match values.first() {
                Some(value) => Ok(Some(value.into_py_any(py)?)),
                None => {
                    if let Some(default_value) = default {
                        Ok(Some(default_value.into_any().unbind()))
                    } else {
                        Ok(None)
                    }
                }
            },
            _ => {
                if let Some(default_value) = default {
                    Ok(Some(default_value.into_any().unbind()))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn get_list(&self, key: &str) -> Vec<String> {
        match self.params.get(key) {
            Some(values) => values.clone(),
            None => vec![],
        }
    }

    pub fn set(&self, key: String, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut q = QueryParams {
            params: self.params.clone(),
        };

        q.params.insert(key, vec![primitive_value_to_str(value)?]);
        Ok(q)
    }

    pub fn add(&self, key: &str, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let mut q = QueryParams {
            params: self.params.clone(),
        };

        let value = primitive_value_to_str(value)?;
        q.params.entry(key.to_string()).or_default().push(value);
        Ok(q)
    }

    pub fn remove(&self, key: &str) -> Self {
        let mut q = QueryParams {
            params: self.params.clone(),
        };

        q.params.shift_remove(key);
        q
    }

    #[pyo3(signature = (params = None))]
    pub fn merge(&self, params: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        if let Some(params) = params {
            let mut new_params = self.params.clone();
            let other = QueryParams::from_pyany(params)?;
            new_params.extend(other.params);
            Ok(QueryParams { params: new_params })
        } else {
            Ok(self.clone())
        }
    }

    pub fn __getitem__(&self, key: &str) -> PyResult<String> {
        match self.params.get(key) {
            Some(values) if !values.is_empty() => Ok(values[0].clone()),
            _ => Err(PyKeyError::new_err(format!("Key '{}' not found.", key))),
        }
    }

    pub fn __contains__(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    pub fn __iter__(&self) -> QueryParamsKeysIterator {
        QueryParamsKeysIterator {
            params: self.keys().into_iter(),
        }
    }

    pub fn __len__(&self) -> usize {
        self.params.len()
    }

    pub fn __bool__(&self) -> bool {
        !self.params.is_empty()
    }

    pub fn __repr__(&self) -> String {
        format!("QueryParams('{}')", self)
    }

    #[allow(unused_variables)]
    #[pyo3(signature = (params = None))]
    pub fn update(&self, params: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        Err(PyRuntimeError::new_err(
            "QueryParams are immutable since 0.18.0.  Use `q = q.merge(...)` to create an updated copy.",
        ))
    }

    #[allow(unused_variables)]
    pub fn __setitem__(&self, key: String, value: String) -> PyResult<()> {
        Err(PyRuntimeError::new_err(
            "QueryParams are immutable since 0.18.0. Use `q = q.set(key, value)` to create an updated copy.",
        ))
    }
}

impl QueryParams {
    fn from_str(s: &str) -> Self {
        let mut params: IndexMap<String, Vec<String>> = IndexMap::new();
        if s.is_empty() {
            return QueryParams {
                params: IndexMap::new(),
            };
        }
        for pair in s.split('&') {
            let pair: Vec<&str> = pair.split("=").collect();
            match pair.len() {
                2 => {
                    params
                        .entry(pair[0].to_string())
                        .or_default()
                        .push(pair[1].to_string());
                }
                1 => {
                    params
                        .entry(pair[0].to_string())
                        .or_default()
                        .push("".to_string());
                }
                _ => {}
            }
        }
        QueryParams { params }
    }

    fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut params: IndexMap<String, Vec<String>> = IndexMap::with_capacity(dict.len());
        for (key, value) in dict.iter() {
            let value = if let Ok(value) = value.downcast::<PyList>() {
                let mut values = Vec::with_capacity(value.len());
                for item in value {
                    values.push(primitive_value_to_str(&item)?);
                }
                values
            } else if let Ok(value) = value.downcast::<PyTuple>() {
                let mut values = Vec::with_capacity(value.len());
                for item in value {
                    values.push(primitive_value_to_str(&item)?);
                }
                values
            } else {
                vec![primitive_value_to_str(&value)?]
            };

            params.insert(key.extract::<String>()?, value);
        }
        Ok(QueryParams { params })
    }

    fn from_pyany(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if obj.is_none() {
            Ok(QueryParams {
                params: IndexMap::new(),
            })
        } else if let Ok(obj) = obj.extract::<QueryParams>() {
            Ok(QueryParams {
                params: obj.params.clone(),
            })
        } else if let Ok(obj) = obj.extract::<&str>() {
            Ok(QueryParams::from_str(&obj))
        } else if let Ok(obj) = obj.extract::<&[u8]>() {
            Ok(QueryParams::from_str(std::str::from_utf8(obj)?))
        } else if let Ok(obj) = obj.downcast::<PyList>() {
            let mut params: IndexMap<String, Vec<String>> = IndexMap::with_capacity(obj.len());
            for item in obj.iter() {
                let (key, value) = item.extract::<(String, String)>()?;
                params.entry(key).or_default().push(value);
            }
            Ok(QueryParams { params })
        } else if let Ok(obj) = obj.downcast::<PyTuple>() {
            let mut params: IndexMap<String, Vec<String>> = IndexMap::with_capacity(obj.len());
            for item in obj.iter() {
                let (key, value) = item.extract::<(String, String)>()?;
                params.entry(key).or_default().push(value);
            }
            Ok(QueryParams { params })
        } else {
            QueryParams::from_pydict(obj.downcast::<PyDict>()?)
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct QueryParamsKeysIterator {
    params: IntoIter<String>,
}

#[pymethods]
impl QueryParamsKeysIterator {
    pub fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    pub fn __next__(&mut self) -> Option<String> {
        self.params.next()
    }
}

impl PartialEq for QueryParams {
    fn eq(&self, other: &Self) -> bool {
        let mut this = self.multi_items();
        let mut other = other.multi_items();
        this.sort();
        other.sort();
        this == other
    }
}

impl Eq for QueryParams {}

impl std::fmt::Display for QueryParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Vec::with_capacity(self.params.len());
        for (key, value) in &self.params {
            for value in value {
                result.push(format!("{}={}", urlencode(key), urlencode(value)));
            }
        }
        result.join("&");
        write!(f, "{}", result.join("&"))
    }
}

impl Hash for QueryParams {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}
