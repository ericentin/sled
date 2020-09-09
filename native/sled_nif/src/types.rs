use std::ops::Deref;

use rustler::{resource, Env, NifStruct, NifUnitEnum, NifUntaggedEnum, ResourceArc};

use sled::{Config, Db, Tree};

#[derive(NifUnitEnum)]
enum Mode {
    LowSpace,
    HighThroughput,
}

impl From<Mode> for sled::Mode {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::LowSpace => sled::Mode::LowSpace,
            Mode::HighThroughput => sled::Mode::HighThroughput,
        }
    }
}

#[derive(NifStruct)]
#[module = "Sled.Config.Options"]
pub struct SledConfigOptions {
    path: Option<String>,
    cache_capacity: Option<u64>,
    mode: Option<Mode>,
    use_compression: Option<bool>,
    compression_factor: Option<i32>,
    temporary: Option<bool>,
    create_new: Option<bool>,
    print_profile_on_drop: Option<bool>,
}

struct SledConfigResource(Config);

impl SledConfigResource {
    fn with_opts(opts: SledConfigOptions) -> SledConfigResource {
        SledConfigResource(Config::new())
            .set(&Config::path, opts.path)
            .set(&Config::cache_capacity, opts.cache_capacity)
            .set(&Config::mode, opts.mode.map(&sled::Mode::from))
            .set(&Config::use_compression, opts.use_compression)
            .set(&Config::compression_factor, opts.compression_factor)
            .set(&Config::temporary, opts.temporary)
            .set(&Config::create_new, opts.create_new)
            .set(&Config::print_profile_on_drop, opts.print_profile_on_drop)
    }

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

#[derive(NifStruct)]
#[module = "Sled.Config"]
pub struct SledConfig {
    r#ref: ResourceArc<SledConfigResource>,
}

impl SledConfig {
    pub fn with_opts(opts: SledConfigOptions) -> SledConfig {
        SledConfig {
            r#ref: ResourceArc::new(SledConfigResource::with_opts(opts)),
        }
    }
}

impl Deref for SledConfig {
    type Target = Config;

    fn deref(&self) -> &Config {
        &self.r#ref.0
    }
}

struct SledDbResource(Db);

#[derive(NifStruct)]
#[module = "Sled"]
pub struct SledDb {
    r#ref: ResourceArc<SledDbResource>,
    path: String,
}

impl SledDb {
    pub fn with_db_and_path(db: Db, path: String) -> SledDb {
        SledDb {
            r#ref: ResourceArc::new(SledDbResource(db)),
            path,
        }
    }
}

impl Deref for SledDb {
    type Target = Db;

    fn deref(&self) -> &Db {
        &self.r#ref.0
    }
}

struct SledTreeResource(Tree);

#[derive(NifStruct)]
#[module = "Sled.Tree"]
pub struct SledTree {
    r#ref: ResourceArc<SledTreeResource>,
    db: SledDb,
    name: String,
}

impl SledTree {
    pub fn with_tree_db_and_name(tree: Tree, db: SledDb, name: String) -> SledTree {
        SledTree {
            r#ref: ResourceArc::new(SledTreeResource(tree)),
            db,
            name,
        }
    }
}

#[derive(NifUntaggedEnum)]
pub enum SledDbTree {
    Default(SledDb),
    Tenant(SledTree),
}

impl Deref for SledDbTree {
    type Target = Tree;

    fn deref(&self) -> &Tree {
        match &self {
            SledDbTree::Default(db) => &*db,
            SledDbTree::Tenant(tree) => &tree.r#ref.0,
        }
    }
}

pub fn on_load(env: Env) -> bool {
    resource!(SledConfigResource, env);
    resource!(SledDbResource, env);
    resource!(SledTreeResource, env);
    true
}
