use std::error;
use serde_json::{Value};
use serde_json::value::{Index};

// FIXME: This only works for homogenous indices atm
pub fn get_in<'a, I: Index>(v: &'a Value, indices: &'a [I]) -> Option<&'a Value> {
    if let Some(index) = indices.get(0) {
        if let Some(index2) = indices.get(1) {
            return get_in(v.get(index).unwrap(), &indices[1..]);
        } else {
            return v.get(index);
        }
    } else {
        return None;
    }
}

// FIXME: Shouldn't clone, should move in
// FIXME: use serde::{Serialize, Deserialize}
pub fn serde_json_value_to_vec(v: Value) -> Result<Vec<Value>, Box<dyn error::Error>> {
    let mut out = Vec::new();
    let mut i = 0;
    loop {
        if let Some(v) = v.get(i) {
            out.push(v.clone());
            i = i + 1;
        } else {
            break;
        }
    }
    Ok(out)
}
