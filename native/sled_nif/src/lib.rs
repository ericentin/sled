#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate lazy_static;

mod transactional;
mod types;

use rustler::{init, nif, types::atom::ok, Atom, Binary, Env, Error, NifResult, OwnedBinary, Term};
use sled::IVec;

use transactional::*;
use types::*;

#[nif]
fn sled_config_new(opts: SledConfigOptions) -> NifResult<SledConfig> {
    Ok(SledConfig::with_opts(opts))
}

#[nif(schedule = "DirtyIo")]
fn sled_config_open(config: SledConfig) -> NifResult<SledDb> {
    rustler_result_from_sled(config.open())
        .map(|db| SledDb::with_db_and_path(db, String::from(config.path.to_string_lossy())))
}

#[nif(schedule = "DirtyIo")]
fn sled_open(path: String) -> NifResult<SledDb> {
    rustler_result_from_sled(sled::open(path.clone())).map(|db| SledDb::with_db_and_path(db, path))
}

#[nif(schedule = "DirtyIo")]
fn sled_db_checksum(db: SledDb) -> NifResult<u32> {
    rustler_result_from_sled(db.checksum())
}

#[nif(schedule = "DirtyIo")]
fn sled_size_on_disk(db: SledDb) -> NifResult<u64> {
    rustler_result_from_sled(db.size_on_disk())
}

#[nif(schedule = "DirtyIo")]
fn sled_was_recovered(db: SledDb) -> bool {
    db.was_recovered()
}

#[nif(schedule = "DirtyIo")]
fn sled_generate_id(db: SledDb) -> NifResult<u64> {
    rustler_result_from_sled(db.generate_id())
}

#[nif(schedule = "DirtyIo")]
fn sled_export(env: Env, db: SledDb) -> NifResult<SledExport> {
    let export = db.export();
    let mut result = SledExport::with_capacity(export.len());

    for (collection_type, collection_name, collection_iter) in export {
        let collection_type_bin = try_binary_from(env, &collection_type)?;
        let collection_name_bin = try_binary_from(env, &collection_name)?;

        let (lower_size_bound, maybe_upper_size_bound) = collection_iter.size_hint();
        let mut collection_iter_result =
            Vec::with_capacity(maybe_upper_size_bound.unwrap_or(lower_size_bound));

        for collection_iter_item in collection_iter {
            let mut collection_iter_item_result = Vec::with_capacity(collection_iter_item.len());

            for collection_iter_item_item in collection_iter_item {
                collection_iter_item_result.push(try_binary_from(env, &collection_iter_item_item)?)
            }

            collection_iter_result.push(collection_iter_item_result)
        }

        result.push((
            collection_type_bin,
            collection_name_bin,
            collection_iter_result,
        ))
    }

    Ok(result)
}

#[nif(schedule = "DirtyIo")]
fn sled_import(db: SledDb, export: SledExport) -> Atom {
    let mut result = Vec::with_capacity(export.len());

    for (collection_type, collection_name, collection_items) in export {
        let mut collection_items_result = Vec::with_capacity(collection_items.len());

        for collection_item in collection_items {
            let mut collection_items_items_result = Vec::with_capacity(collection_item.len());

            for collection_item_item in collection_item {
                collection_items_items_result.push(Vec::from(&collection_item_item[..]))
            }

            collection_items_result.push(collection_items_items_result)
        }

        result.push((
            Vec::from(&collection_type[..]),
            Vec::from(&collection_name[..]),
            collection_items_result.into_iter(),
        ))
    }

    db.import(result);

    ok()
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_open(db: SledDb, name: String) -> NifResult<SledTree> {
    rustler_result_from_sled(db.open_tree(name.clone()))
        .map(|tree| SledTree::with_tree_db_and_name(tree, db, name))
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_drop(db: SledDb, name: String) -> NifResult<bool> {
    rustler_result_from_sled(db.drop_tree(name))
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_names(env: Env, db: SledDb) -> NifResult<Vec<Binary>> {
    let tree_names = db.tree_names();
    let mut result = Vec::with_capacity(tree_names.len());

    for tree_name in tree_names {
        result.push(try_binary_from(env, &tree_name)?)
    }

    Ok(result)
}

#[nif(schedule = "DirtyIo")]
fn sled_checksum(tree: SledDbTree) -> NifResult<u32> {
    rustler_result_from_sled(tree.checksum())
}

#[nif(schedule = "DirtyIo")]
fn sled_flush(tree: SledDbTree) -> NifResult<usize> {
    rustler_result_from_sled(tree.flush())
}

#[nif(schedule = "DirtyIo")]
fn sled_insert<'a>(
    env: Env<'a>,
    tree: SledDbTree,
    k: Binary,
    v: Binary,
) -> NifResult<Option<Binary<'a>>> {
    try_binary_result_from_sled(env, tree.insert(&k[..], &v[..]))
}

#[nif(schedule = "DirtyIo")]
fn sled_get<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> NifResult<Option<Binary<'a>>> {
    try_binary_result_from_sled(env, tree.get(&k[..]))
}

#[nif(schedule = "DirtyIo")]
fn sled_remove<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> NifResult<Option<Binary<'a>>> {
    try_binary_result_from_sled(env, tree.remove(&k[..]))
}

#[nif(schedule = "DirtyIo")]
fn sled_compare_and_swap<'a>(
    env: Env<'a>,
    tree: SledDbTree,
    k: Binary,
    old: Option<Binary<'a>>,
    new: Option<Binary<'a>>,
) -> NifResult<Result<(), (Option<Binary<'a>>, Option<Binary<'a>>)>> {
    let result = tree.compare_and_swap(
        &k[..],
        old.map(|old| old.as_slice()),
        new.map(|new| new.as_slice()),
    );

    match rustler_result_from_sled(result)? {
        Ok(()) => Ok(Ok(())),
        Err(err) => {
            let current_bin = match err.current {
                Some(v) => Some(try_binary_from(env, &v[..])?),
                None => None,
            };
            let proposed_bin = match err.proposed {
                Some(v) => Some(try_binary_from(env, &v[..])?),
                None => None,
            };
            Ok(Err((current_bin, proposed_bin)))
        }
    }
}

#[nif(schedule = "DirtyIo")]
fn sled_transaction(env: Env, tree: SledDbTree) -> SledTransactionalTree {
    transaction_new(env, tree)
}

#[nif(schedule = "DirtyIo")]
fn sled_transaction_close(env: Env, tx_tree: SledTransactionalTree, req_ref: Binary) {
    transaction_close(env, tx_tree, req_ref).unwrap()
}

#[nif(schedule = "DirtyIo")]
fn sled_transaction_insert(
    env: Env,
    tx_tree: SledTransactionalTree,
    req_ref: Binary,
    k: Binary,
    v: Binary,
) {
    transaction_insert(env, tx_tree, req_ref, k, v).unwrap()
}

fn on_load(env: Env, _info: Term) -> bool {
    types::on_load(env)
}

fn rustler_result_from_sled<T>(r: sled::Result<T>) -> NifResult<T> {
    r.map_err(|err| raise_term_from_string(format!("sled::Error::{:?}", err)))
}

fn raise_term_from_string(error: String) -> Error {
    Error::RaiseTerm(Box::new(error))
}

fn try_binary_result_from_sled(
    env: Env,
    r: sled::Result<Option<IVec>>,
) -> NifResult<Option<Binary>> {
    match rustler_result_from_sled(r) {
        Ok(Some(v)) => try_binary_from(env, &v).map(&Some),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

fn try_binary_from<'a>(env: Env<'a>, v: &[u8]) -> NifResult<Binary<'a>> {
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

init! {
    "Elixir.Sled.Native",
    [
        sled_config_new,
        sled_config_open,
        sled_open,
        sled_tree_open,
        sled_tree_drop,
        sled_tree_names,
        sled_db_checksum,
        sled_size_on_disk,
        sled_was_recovered,
        sled_generate_id,
        sled_export,
        sled_import,
        sled_checksum,
        sled_flush,
        sled_insert,
        sled_get,
        sled_remove,
        sled_compare_and_swap,
        sled_transaction,
        sled_transaction_close,
        sled_transaction_insert,
    ],
    load = on_load
}
