#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

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
    pub r#ref: ResourceArc<SledConfigArc>,
}

struct SledConfigArc(Config);

impl SledConfigArc {
    fn set<T, F: Fn(Config, T) -> Config>(mut self, setter: F, value: Option<T>) -> SledConfigArc {
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
    let config = SledConfigArc(Config::new())
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

struct SledDbArc(Db);

#[derive(NifStruct)]
#[module = "Sled"]
struct SledDb {
    pub r#ref: ResourceArc<SledDbArc>,
    pub path: String,
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_config_open(config: SledConfig) -> Result<SledDb, Error> {
    do_sled_open(
        config.r#ref.0.open(),
        String::from(config.r#ref.0.path.to_string_lossy()),
    )
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_open(path: String) -> Result<SledDb, Error> {
    do_sled_open(sled::open(path.clone()), path)
}

fn do_sled_open(result: sled::Result<Db>, path: String) -> Result<SledDb, Error> {
    match result {
        Ok(db) => Ok(SledDb {
            r#ref: ResourceArc::new(SledDbArc(db)),
            path,
        }),
        Err(err) => wrap_sled_err(&err),
    }
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_db_checksum(db: SledDb) -> Result<u32, Error> {
    wrap_result(db.r#ref.0.checksum())
}

struct SledTreeArc(Tree);

#[derive(NifStruct)]
#[module = "Sled.Tree"]
struct SledTree {
    pub r#ref: ResourceArc<SledTreeArc>,
    pub db: SledDb,
    pub name: String,
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_tree_open(db: SledDb, name: String) -> Result<SledTree, Error> {
    match db.r#ref.0.open_tree(name.clone()) {
        Ok(tree) => Ok(SledTree {
            r#ref: ResourceArc::new(SledTreeArc(tree)),
            db,
            name,
        }),
        Err(err) => wrap_sled_err(&err),
    }
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

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_checksum(tree: SledDbTree) -> Result<u32, Error> {
    wrap_result(tree.checksum())
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_insert<'a>(
    env: Env<'a>,
    tree: SledDbTree,
    k: Binary,
    v: Binary,
) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.insert(&k[..], &v[..]))
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_get<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.get(&k[..]))
}

#[allow(clippy::needless_pass_by_value)]
#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_remove<'a>(env: Env<'a>, tree: SledDbTree, k: Binary) -> Result<Option<Binary<'a>>, Error> {
    result_to_binary(env, tree.remove(&k[..]))
}

fn wrap_result<T>(r: Result<T, sled::Error>) -> Result<T, Error> {
    match r {
        Ok(v) => Ok(v),
        Err(err) => wrap_sled_err(&err),
    }
}

fn result_to_binary(
    env: Env,
    r: Result<Option<IVec>, sled::Error>,
) -> Result<Option<Binary>, Error> {
    match wrap_result(r) {
        Ok(Some(v)) => ivec_to_binary(env, &v),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}

fn ivec_to_binary<'a>(env: Env<'a>, v: &IVec) -> Result<Option<Binary<'a>>, Error> {
    match OwnedBinary::new(v.len()) {
        Some(mut owned_binary) => {
            owned_binary.as_mut_slice().copy_from_slice(&v);
            Ok(Some(owned_binary.release(env)))
        }
        None => wrap_err(String::from(
            "failed to allocate OwnedBinary for result value",
        )),
    }
}

fn wrap_sled_err<T>(err: &sled::Error) -> Result<T, Error> {
    wrap_err(format!("sled::Error::{:?}", err))
}

fn wrap_err<T>(err: String) -> Result<T, Error> {
    Err(Error::RaiseTerm(Box::new(err)))
}

fn on_load(env: Env, _info: Term) -> bool {
    resource!(SledConfigArc, env);
    resource!(SledDbArc, env);
    resource!(SledTreeArc, env);
    true
}

init! {
    "Elixir.Sled.Native",
    [
        sled_config_new,
        sled_config_open,
        sled_open,
        sled_tree_open,
        sled_db_checksum,
        sled_checksum,
        sled_insert,
        sled_get,
        sled_remove
    ],
    load = on_load
}
