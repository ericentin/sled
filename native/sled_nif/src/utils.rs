use rustler::{Binary, Env, Error, NifResult, OwnedBinary};

use sled::IVec;

pub fn rustler_result_from_sled<T>(r: sled::Result<T>) -> NifResult<T> {
    r.map_err(|err| raise_term_from_string(format!("sled::Error::{:?}", err)))
}

pub fn raise_term_from_string(error: String) -> Error {
    Error::RaiseTerm(Box::new(error))
}

pub fn try_binary_result_from_sled(
    env: Env,
    r: sled::Result<Option<IVec>>,
) -> NifResult<Option<Binary>> {
    match rustler_result_from_sled(r) {
        Ok(Some(v)) => try_binary_from(env, &v).map(&Some),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn try_binary_from<'a>(env: Env<'a>, v: &[u8]) -> NifResult<Binary<'a>> {
    match OwnedBinary::new(v.len()) {
        Some(mut owned_binary) => {
            owned_binary.as_mut_slice().copy_from_slice(&v);
            Ok(owned_binary.release(env))
        }
        None => Err(raise_term_from_string(String::from(
            "failed to allocate OwnedBinary for result value",
        ))),
    }
}
