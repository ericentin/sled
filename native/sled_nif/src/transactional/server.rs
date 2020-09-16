use crossbeam_channel::Receiver;
use rustler::{
    types::atom::nil, types::atom::ok, Binary, Encoder, Env, LocalPid, OwnedBinary, Term,
};
use sled::{
    transaction::abort, transaction::ConflictableTransactionError, transaction::TransactionError,
    IVec, Tree,
};

use crate::types::{SledTransactionalTreeCommand, SledTransactionalTreeRequest};

mod atoms {
    rustler::atoms! {
        sled_transaction,
        abort,
        storage
    }
}

pub fn transaction_server<'a>(
    env: Env<'a>,
    tree: &Tree,
    r: &Receiver<SledTransactionalTreeRequest>,
) -> Term<'a> {
    tree.transaction(move |tx_tree| loop {
        match wrap_recv(env, r) {
            Ok((caller, req_ref, command)) => match command {
                SledTransactionalTreeCommand::Insert(key, value) => {
                    match tx_tree.insert(key, value)? {
                        Some(v) => {
                            let result = try_ivec_to_binary(env, v)?;
                            reply(env, caller, req_ref, result)
                        }
                        None => reply(env, caller, req_ref, nil()),
                    }
                }
                SledTransactionalTreeCommand::Flush => {
                    tx_tree.flush();
                    reply(env, caller, req_ref, ok())
                }
                SledTransactionalTreeCommand::Close => {
                    break Ok(req_ref);
                }
            },
            Err(err) => break Err(err),
        }
    })
    .map_err(|tx_error| match tx_error {
        TransactionError::Abort(term) => (atoms::abort(), term),
        TransactionError::Storage(err) => (atoms::storage(), format!("sled::result::{:?}", err)),
    })
    .encode(env)
}

fn wrap_recv<'a>(
    env: Env<'a>,
    r: &Receiver<SledTransactionalTreeRequest>,
) -> Result<(LocalPid, Term<'a>, SledTransactionalTreeCommand), ConflictableTransactionError<String>>
{
    match r.recv() {
        Ok((caller, req_ref, command)) => match env.binary_to_term(&req_ref[..]) {
            Some((req_ref, _)) => Ok((caller, req_ref, command)),
            None => abort(String::from("failed to decode req_ref")),
        },
        Err(err) => abort(format!("crossbeam-channel::err::{:?}", err)),
    }
}

fn reply<T>(env: Env, caller: LocalPid, req_ref: Term, value: T)
where
    T: Encoder,
{
    env.send(
        &caller,
        (atoms::sled_transaction(), req_ref, value).encode(env),
    )
}

fn try_ivec_to_binary(env: Env, v: IVec) -> Result<Binary, ConflictableTransactionError<String>> {
    match OwnedBinary::new(v.len()) {
        Some(mut owned_binary) => {
            owned_binary.as_mut_slice().copy_from_slice(&v);
            Ok(owned_binary.release(env))
        }
        None => abort(String::from(
            "failed to allocate OwnedBinary for result value",
        )),
    }
}
