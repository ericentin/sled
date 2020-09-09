#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::ops::Deref;

use rustler::{
    init, nif, resource, Binary, Env, Error, NifStruct, NifUnitEnum, NifUntaggedEnum, OwnedBinary,
    ResourceArc, Term,
};

use sled::{Config, Db, IVec, Tree};

#[derive(NifStruct)]
#[module = "Sled.Config.Options"]
struct SledConfigOptions {
    pub path: Option<String>,
    pub cache_capacity: Option<u64>,
    pub mode: Option<Mode>,
    pub use_compression: Option<bool>,
    pub compression_factor: Option<i32>,
    pub temporary: Option<bool>,
    pub create_new: Option<bool>,
    pub print_profile_on_drop: Option<bool>,
}

#[derive(NifUnitEnum)]
enum Mode {
    LowSpace,
    HighThroughput,
}

#[derive(NifStruct)]
#[module = "Sled.Config"]
struct SledConfig {
    pub r#ref: ResourceArc<SledConfigResource>,
}

struct SledConfigResource(Config);

impl SledConfigResource {
    fn set<T, F: Fn(Config, T) -> Config>(
        mut self,
        setter: F,
        value: Option<T>,
    ) -> SledConfigResource {
        match value {
            Some(value) => {
                self.0 = setter(self.0, value);
                self
            }
            None => self,
        }
    }
}

fn mode_to_rust(mode: Option<Mode>) -> Option<sled::Mode> {
    mode.map(|mode| match mode {
        Mode::LowSpace => sled::Mode::LowSpace,
        Mode::HighThroughput => sled::Mode::HighThroughput,
    })
}

#[nif]
fn sled_config_new(opts: SledConfigOptions) -> Result<SledConfig, Error> {
    let config = SledConfigResource(Config::new())
        .set(&Config::path, opts.path)
        .set(&Config::cache_capacity, opts.cache_capacity)
        .set(&Config::mode, mode_to_rust(opts.mode))
        .set(&Config::use_compression, opts.use_compression)
        .set(&Config::compression_factor, opts.compression_factor)
        .set(&Config::temporary, opts.temporary)
        .set(&Config::create_new, opts.create_new)
        .set(&Config::print_profile_on_drop, opts.print_profile_on_drop);

    Ok(SledConfig {
        r#ref: ResourceArc::new(config),
    })
}

struct SledDbResource(Db);

#[derive(NifStruct)]
#[module = "Sled"]
struct SledDb {
    pub r#ref: ResourceArc<SledDbResource>,
    pub path: String,
}

#[nif(schedule = "DirtyIo")]
fn sled_config_open(config: SledConfig) -> Result<SledDb, Error> {
    do_sled_open(
        config.r#ref.0.open(),
        String::from(config.r#ref.0.path.to_string_lossy()),
    )
}

#[nif(schedule = "DirtyIo")]
fn sled_open(path: String) -> Result<SledDb, Error> {
    do_sled_open(sled::open(path.clone()), path)
}

fn do_sled_open(result: sled::Result<Db>, path: String) -> Result<SledDb, Error> {
    wrap_result(result).map(|db| SledDb {
        r#ref: ResourceArc::new(SledDbResource(db)),
        path,
    })
}

#[nif(schedule = "DirtyIo")]
fn sled_db_checksum(db: SledDb) -> Result<u32, Error> {
    wrap_result(db.r#ref.0.checksum())
}

#[nif(schedule = "DirtyIo")]
fn sled_size_on_disk(db: SledDb) -> Result<u64, Error> {
    wrap_result(db.r#ref.0.size_on_disk())
}

#[nif(schedule = "DirtyIo")]
fn sled_was_recovered(db: SledDb) -> bool {
    db.r#ref.0.was_recovered()
}

struct SledTreeResource(Tree);

#[derive(NifStruct)]
#[module = "Sled.Tree"]
struct SledTree {
    pub r#ref: ResourceArc<SledTreeResource>,
    pub db: SledDb,
    pub name: String,
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_open(db: SledDb, name: String) -> Result<SledTree, Error> {
    wrap_result(db.r#ref.0.open_tree(name.clone())).map(|tree| SledTree {
        r#ref: ResourceArc::new(SledTreeResource(tree)),
        db,
        name,
    })
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_drop(db: SledDb, name: String) -> Result<bool, Error> {
    wrap_result(db.r#ref.0.drop_tree(name))
}

#[nif(schedule = "DirtyIo")]
fn sled_tree_names(env: Env, db: SledDb) -> Result<Vec<Binary>, Error> {
    let tree_names = db.r#ref.0.tree_names();
    let mut result = Vec::with_capacity(tree_names.len());

    for tree_name in tree_names {
        result.push(ivec_to_binary(env, &tree_name)?)
    }

    Ok(result)
}

#[derive(NifUntaggedEnum)]
enum SledDbTree {
    Default(SledDb),
    Tenant(SledTree),
}

impl Deref for SledDbTree {
    type Target = Tree;

    fn deref(&self) -> &Tree {
        match &self {
            SledDbTree::Default(db) => &*db.r#ref.0,
            SledDbTree::Tenant(tree) => &tree.r#ref.0,
        }
    }
}

#[nif(schedule = "DirtyIo")]
fn sled_checksum(tree: SledDbTree) -> Result<u32, Error> {
    wrap_result(tree.checksum())
}

#[nif(schedule = "DirtyIo")]
fn sled_flush(tree: SledDbTree) -> Result<usize, Error> {
    wrap_result(tree.flush())
}

#[nif(schedule = "DirtyIo")]
fn sled_insert<'a>(
    env: Env<'a>,
    tree: SledDbTree,
    k: Binary,
    v: Binary,
) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.insert(&k[..], &v[..]))
}

#[nif(schedule = "DirtyIo")]
fn sled_get<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.get(&k[..]))
}

#[nif(schedule = "DirtyIo")]
fn sled_remove<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.remove(&k[..]))
}

fn wrap_result<T>(r: Result<T, sled::Error>) -> Result<T, Error> {
    r.map_err(|err| wrap_sled_err(&err))
}

fn result_to_binary(
    env: Env,
    r: Result<Option<IVec>, sled::Error>,
) -> Result<Option<Binary>, Error> {
    match wrap_result(r) {
        Ok(Some(v)) => ivec_to_binary(env, &v).map(&Some),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

fn ivec_to_binary<'a>(env: Env<'a>, v: &IVec) -> Result<Binary<'a>, Error> {
    match OwnedBinary::new(v.len()) {
        Some(mut owned_binary) => {
            owned_binary.as_mut_slice().copy_from_slice(&v);
            Ok(owned_binary.release(env))
        }
        None => Err(wrap_err(String::from(
            "failed to allocate OwnedBinary for result value",
        ))),
    }
}

fn wrap_sled_err(err: &sled::Error) -> Error {
    wrap_err(format!("sled::Error::{:?}", err))
}

fn wrap_err(err: String) -> Error {
    Error::RaiseTerm(Box::new(err))
}

fn on_load(env: Env, _info: Term) -> bool {
    resource!(SledConfigResource, env);
    resource!(SledDbResource, env);
    resource!(SledTreeResource, env);
    true
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
        sled_checksum,
        sled_flush,
        sled_insert,
        sled_get,
        sled_remove
    ],
    load = on_load
}
