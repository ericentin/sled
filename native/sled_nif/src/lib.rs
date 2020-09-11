#![warn(clippy::all, clippy::pedantic)]

mod types;
mod utils;

use rustler::{init, nif, types::atom::ok, Atom, Binary, Env, NifResult, Term};

use types::*;
use utils::*;

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

fn on_load(env: Env, _info: Term) -> bool {
    types::on_load(env)
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
        sled_remove
    ],
    load = on_load
}
