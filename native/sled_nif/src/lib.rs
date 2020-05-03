#![warn(clippy::all)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use rustler::{
    init, nif, resource, types::atom, Atom, Binary, Env, Error, Error::BadArg, NifStruct,
    OwnedBinary, ResourceArc, Term,
};
use sled::{Config, Db, SegmentMode};
use std::path::PathBuf;

mod atoms {
    rustler::atoms! {
        linear,
        gc,
    }
}

#[derive(NifStruct)]
#[module = "Sled.Config.Options"]
struct SledConfigOptions {
    pub path: Option<String>,
    pub flush_every_ms: Option<(bool, Option<u64>)>,
    pub temporary: Option<bool>,
    pub create_new: Option<bool>,
    pub cache_capacity: Option<u64>,
    pub print_profile_on_drop: Option<bool>,
    pub use_compression: Option<bool>,
    pub compression_factor: Option<i32>,
    pub snapshot_after_ops: Option<u64>,
    pub segment_cleanup_threshold: Option<u8>,
    pub segment_cleanup_skew: Option<usize>,
    pub segment_mode: Atom,
    pub snapshot_path: Option<(bool, Option<String>)>,
    pub idgen_persist_interval: Option<u64>,
    pub read_only: Option<bool>,
}

struct SledConfig(Config);

struct SledDb(Db);

#[nif]
fn sled_config_new(config_options: SledConfigOptions) -> Result<ResourceArc<SledConfig>, Error> {
    let flush_every_ms = flush_every_ms_to_rust(config_options.flush_every_ms)?;
    let segment_mode = segment_mode_to_rust(config_options.segment_mode)?;
    let snapshot_path = snapshot_path_to_rust(config_options.snapshot_path)?;

    let mut config = Config::new();

    macro_rules! configure {
        ($(($setter:ident, $value:expr)),+) => {{
            $(
                config = match $value {
                    Some(value) => config.$setter(value),
                    None => config
                };
            )*
            config
        }}
    }

    Ok(ResourceArc::new(SledConfig(configure!(
        (path, config_options.path),
        (flush_every_ms, flush_every_ms),
        (temporary, config_options.temporary),
        (create_new, config_options.create_new),
        (cache_capacity, config_options.cache_capacity),
        (print_profile_on_drop, config_options.print_profile_on_drop),
        (use_compression, config_options.use_compression),
        (compression_factor, config_options.compression_factor),
        (snapshot_after_ops, config_options.snapshot_after_ops),
        (
            segment_cleanup_threshold,
            config_options.segment_cleanup_threshold
        ),
        (segment_cleanup_skew, config_options.segment_cleanup_skew),
        (segment_mode, segment_mode),
        (snapshot_path, snapshot_path),
        (
            idgen_persist_interval,
            config_options.idgen_persist_interval
        ),
        (read_only, config_options.read_only)
    ))))
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_config_open(config: ResourceArc<SledConfig>) -> Result<(Atom, ResourceArc<SledDb>), Error> {
    do_sled_open(config.0.open())
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_open(path: String) -> Result<(Atom, ResourceArc<SledDb>), Error> {
    do_sled_open(sled::open(path))
}

fn do_sled_open(result: sled::Result<Db>) -> Result<(Atom, ResourceArc<SledDb>), Error> {
    match result {
        Ok(db) => Ok((atom::ok(), ResourceArc::new(SledDb(db)))),
        Err(err) => wrap_err(err),
    }
}

#[nif]
fn sled_config_inspect(config: ResourceArc<SledConfig>) -> Result<String, Error> {
    Ok(format!("{:?}", config.0))
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_insert(resource: ResourceArc<SledDb>, k: Binary, v: Binary) -> Result<Atom, Error> {
    resource.0.insert(k.as_slice(), v.as_slice()).unwrap();

    Ok(atom::ok())
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_get<'a>(
    env: Env<'a>,
    resource: ResourceArc<SledDb>,
    k: Binary,
) -> Result<(Atom, Option<Binary<'a>>), Error> {
    let SledDb(db) = &*resource;
    match db.get(k.as_slice()) {
        Ok(Some(v)) => match OwnedBinary::new(v.len()) {
            Some(mut owned_binary) => {
                owned_binary.as_mut_slice().copy_from_slice(v.as_ref());
                Ok((atom::ok(), Some(owned_binary.release(env))))
            }
            None => Err(Error::Term(Box::new(
                "Failed to allocated OwnedBinary for result value.",
            ))),
        },
        Ok(None) => Ok((atom::ok(), None)),
        Err(err) => wrap_err(err),
    }
}

#[allow(clippy::option_option)]
fn flush_every_ms_to_rust(
    value: Option<(bool, Option<u64>)>,
) -> Result<Option<Option<u64>>, Error> {
    match value {
        Some((true, Some(ms))) => Ok(Some(Some(ms))),
        Some((false, None)) => Ok(Some(None)),
        Some((true, None)) => Err(BadArg),
        Some((false, _)) => Err(BadArg),
        None => Ok(None),
    }
}

fn segment_mode_to_rust(segment_mode: Atom) -> Result<Option<SegmentMode>, Error> {
    match segment_mode {
        atom if atom == atoms::linear() => Ok(Some(SegmentMode::Linear)),
        atom if atom == atoms::gc() => Ok(Some(SegmentMode::Gc)),
        atom if atom == atom::nil() => Ok(None),
        _ => Err(BadArg),
    }
}

#[allow(clippy::option_option)]
fn snapshot_path_to_rust(
    value: Option<(bool, Option<String>)>,
) -> Result<Option<Option<PathBuf>>, Error> {
    match value {
        Some((true, Some(snapshot_path))) => Ok(Some(Some(PathBuf::from(snapshot_path)))),
        Some((false, None)) => Ok(Some(None)),
        Some((true, None)) => Err(BadArg),
        Some((false, _)) => Err(BadArg),
        None => Ok(None),
    }
}

fn wrap_err<T>(err: sled::Error) -> Result<T, Error> {
    Err(Error::Term(Box::new(format!("{}", err))))
}

fn on_load(env: Env, _info: Term) -> bool {
    resource!(SledConfig, env);
    resource!(SledDb, env);
    true
}

init! {
    "Elixir.Sled.Native",
    [
        sled_config_new,
        sled_config_open,
        sled_config_inspect,
        sled_open,
        sled_insert,
        sled_get
    ],
    load = on_load
}
