use serde_json::{Value};
use serde_json::value::{Index};

// FIXME: This only works for homogenous indices atm
pub fn get_in<'a, 'b, I: Index>(v: &'a Value, indices: &'b [I]) -> Option<&'a Value> {
    if let Some(index) = indices.get(0) {
        if let Some(_index2) = indices.get(1) {
            return get_in(v.get(index).unwrap(), &indices[1..]);
        } else {
            return v.get(index);
        }
    } else {
        return None;
    }
}
