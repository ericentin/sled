#![warn(clippy::all)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
use rustler::{resource_struct_init, Encoder, Env, Error, OwnedBinary, ResourceArc, Term};
use sled;

mod atoms {
    rustler::rustler_atoms! {
        atom ok;
        atom error;
        atom nil;
        //atom __true__ = "true";
        //atom __false__ = "false";
    }
}

struct SledResource {
    pub t: sled::Db,
}

rustler::rustler_export_nifs! {
    "Elixir.Sled.Native",
    [
        ("sled_open", 1, sled_open),
        ("sled_insert", 3, sled_insert),
        ("sled_get", 2, sled_get)
    ],
    Some(on_load)
}

fn on_load(env: Env, _info: Term) -> bool {
    resource_struct_init!(SledResource, env);
    true
}

fn sled_open<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let db_name: String = args[0].decode()?;

    match sled::open(db_name) {
        Ok(t) => {
            let resource = ResourceArc::new(SledResource { t });
            Ok((atoms::ok(), resource).encode(env))
        }
        Err(_) => Ok(atoms::error().encode(env)),
    }
}

fn sled_insert<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<SledResource> = args[0].decode()?;
    let k: String = args[1].decode()?;
    let v: String = args[2].decode()?;
    resource.t.insert(k.as_bytes(), v.as_bytes()).unwrap();

    Ok(atoms::ok().encode(env))
}

struct SledIVec(sled::IVec);

impl Encoder for SledIVec {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let len = self.0.len();
        let mut bin = OwnedBinary::new(len).unwrap();
        bin.as_mut_slice().copy_from_slice(self.0.as_ref());
        bin.release(env).to_term(env)
    }
}

fn sled_get<'a>(env: Env<'a>, args: &[Term<'a>]) -> Result<Term<'a>, Error> {
    let resource: ResourceArc<SledResource> = args[0].decode()?;
    let k: String = args[1].decode()?;
    match resource.t.get(k.as_bytes()) {
        Ok(Some(v)) => Ok((atoms::ok(), SledIVec(v)).encode(env)),
        Ok(None) => Ok((atoms::ok(), atoms::nil()).encode(env)),
        Err(_inner) => Ok(atoms::error().encode(env)),
    }
}
