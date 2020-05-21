#![warn(clippy::all)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use rustler::{
    init, nif, resource, types::atom, Atom, Binary, Env, Error, Error::BadArg, NifStruct,
    NifUnitEnum, NifUntaggedEnum, OwnedBinary, ResourceArc, Term,
};
use sled::{Config, Db};
use std::path::PathBuf;

mod atoms {
    rustler::atoms! {
        linear,
        gc,
    }
}

#[derive(NifStruct)]
#[module = "Sled.Error"]
struct SledError {
    pub message: String,
    pub __exception__: bool,
}

#[derive(NifUnitEnum)]
enum SegmentMode {
    Linear,
    Gc,
}

#[derive(NifUntaggedEnum)]
enum FlushEveryMsConfig {
    DisabledOrUnset(Atom),
    Set(u64),
}

#[derive(NifUntaggedEnum)]
enum SnapshotPathConfig {
    DisabledOrUnset(Atom),
    Set(String),
}

#[derive(NifStruct)]
#[module = "Sled.Config.Options"]
struct SledConfigOptions {
    pub path: Option<String>,
    pub flush_every_ms: FlushEveryMsConfig,
    pub temporary: Option<bool>,
    pub create_new: Option<bool>,
    pub cache_capacity: Option<u64>,
    pub print_profile_on_drop: Option<bool>,
    pub use_compression: Option<bool>,
    pub compression_factor: Option<i32>,
    pub snapshot_after_ops: Option<u64>,
    pub segment_cleanup_threshold: Option<u8>,
    pub segment_cleanup_skew: Option<usize>,
    pub segment_mode: Option<SegmentMode>,
    pub snapshot_path: SnapshotPathConfig,
    pub idgen_persist_interval: Option<u64>,
    pub read_only: Option<bool>,
}

struct SledConfigArc(Config);

#[derive(NifStruct)]
#[module = "Sled.Config"]
struct SledConfig {
    pub r#ref: ResourceArc<SledConfigArc>,
}

struct SledDbArc(Db);

#[derive(NifStruct)]
#[module = "Sled"]
struct Sled {
    pub r#ref: ResourceArc<SledDbArc>,
}

#[nif]
fn sled_config_new(config_options: SledConfigOptions) -> Result<SledConfig, Error> {
    let flush_every_ms = flush_every_ms_to_rust(config_options.flush_every_ms)?;
    let segment_mode = segment_mode_to_rust(config_options.segment_mode);
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

    Ok(SledConfig {
        r#ref: ResourceArc::new(SledConfigArc(configure!(
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
        ))),
    })
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_config_open(config: SledConfig) -> Result<Sled, Error> {
    do_sled_open(config.r#ref.0.open())
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_open(path: String) -> Result<Sled, Error> {
    do_sled_open(sled::open(path))
}

fn do_sled_open(result: sled::Result<Db>) -> Result<Sled, Error> {
    match result {
        Ok(db) => Ok(Sled {
            r#ref: ResourceArc::new(SledDbArc(db)),
        }),
        Err(err) => wrap_err(err),
    }
}

#[nif]
fn sled_config_inspect(config: SledConfig) -> Result<String, Error> {
    Ok(format!("{:?}", config.r#ref.0))
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_insert(resource: Sled, k: Binary, v: Binary) -> Result<Atom, Error> {
    resource.r#ref.0.insert(k.as_slice(), v.as_slice()).unwrap();

    Ok(atom::ok())
}

#[cfg_attr(feature = "io_uring", nif)]
#[cfg_attr(not(feature = "io_uring"), nif(schedule = "DirtyIo"))]
fn sled_get<'a>(env: Env<'a>, resource: Sled, k: Binary) -> Result<Option<Binary<'a>>, Error> {
    let SledDbArc(db) = &*resource.r#ref;
    match db.get(k.as_slice()) {
        Ok(Some(v)) => match OwnedBinary::new(v.len()) {
            Some(mut owned_binary) => {
                owned_binary.as_mut_slice().copy_from_slice(v.as_ref());
                Ok(Some(owned_binary.release(env)))
            }
            None => Err(Error::RaiseTerm(Box::new(SledError {
                __exception__: true,
                message: String::from("Failed to allocate OTP OwnedBinary for result value."),
            }))),
        },
        Ok(None) => Ok(None),
        Err(err) => wrap_err(err),
    }
}

#[allow(clippy::option_option)]
fn flush_every_ms_to_rust(value: FlushEveryMsConfig) -> Result<Option<Option<u64>>, Error> {
    match value {
        FlushEveryMsConfig::Set(ms) => Ok(Some(Some(ms))),
        FlushEveryMsConfig::DisabledOrUnset(atom) if atom == atom::false_() => Ok(Some(None)),
        FlushEveryMsConfig::DisabledOrUnset(atom) if atom == atom::nil() => Ok(None),
        FlushEveryMsConfig::DisabledOrUnset(_) => Err(BadArg),
    }
}

fn segment_mode_to_rust(segment_mode: Option<SegmentMode>) -> Option<sled::SegmentMode> {
    match segment_mode {
        Some(SegmentMode::Linear) => Some(sled::SegmentMode::Linear),
        Some(SegmentMode::Gc) => Some(sled::SegmentMode::Gc),
        None => None,
    }
}

#[allow(clippy::option_option)]
fn snapshot_path_to_rust(value: SnapshotPathConfig) -> Result<Option<Option<PathBuf>>, Error> {
    match value {
        SnapshotPathConfig::Set(path) => Ok(Some(Some(PathBuf::from(path)))),
        SnapshotPathConfig::DisabledOrUnset(atom) if atom == atom::false_() => Ok(Some(None)),
        SnapshotPathConfig::DisabledOrUnset(atom) if atom == atom::nil() => Ok(None),
        SnapshotPathConfig::DisabledOrUnset(_) => Err(BadArg),
    }
}

fn wrap_err<T>(err: sled::Error) -> Result<T, Error> {
    Err(Error::RaiseTerm(Box::new(SledError {
        __exception__: true,
        message: format!("{}", err),
    })))
}

fn on_load(env: Env, _info: Term) -> bool {
    resource!(SledConfigArc, env);
    resource!(SledDbArc, env);
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
