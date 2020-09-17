use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver};
use crossbeam_utils::atomic::AtomicCell;
use rustler::{spawn, Atom, Binary, Encoder, Env, LocalPid, OwnedBinary, Term, ThreadSpawner};
use sled::transaction::{abort, ConflictableTransactionError, TransactionError};
use sled::{IVec, Tree};

use crate::types::*;

mod atoms {
    rustler::atoms! {
        sled_transaction_status,
        start,
        sled_transaction_complete,
        abort,
        storage,
        sled_transaction_reply
    }
}

pub fn transaction_new(env: Env, tree: SledDbTree) -> SledTransactionalTree {
    let (s, r) = bounded(0);
    let tree_cell = AtomicCell::new(tree.clone());
    let caller = env.pid();

    spawn::<ThreadSpawner, _>(env, move |thread_env: Env| {
        transaction_server(thread_env, &caller, &tree_cell.into_inner(), &r)
    });

    SledTransactionalTree::with_tree_and_sender(tree, s)
}

fn transaction_server<'a>(
    env: Env<'a>,
    caller: &LocalPid,
    tree: &Tree,
    r: &Receiver<SledTransactionalTreeRequest>,
) -> Term<'a> {
    let result = tree
        .transaction(move |tx_tree| {
            send(
                env,
                &caller,
                atoms::sled_transaction_status(),
                atoms::start(),
            );

            loop {
                match r.recv() {
                    Ok(command) => match command.invoke(env, tx_tree)? {
                        SledTransactionalTreeAction::Continue => continue,
                        SledTransactionalTreeAction::Close => break Ok(()),
                        SledTransactionalTreeAction::Abort => {
                            break abort(SledTransactionServerError::UserAbort)
                        }
                    },
                    Err(err) => break abort(SledTransactionServerError::RecvError(err)),
                }
            }
        })
        .map_err(|tx_error| match tx_error {
            TransactionError::Abort(SledTransactionServerError::RecvError(err)) => (
                atoms::abort(),
                Some(format!("crossbeam-channel::err::{:?}", err)),
            ),
            TransactionError::Abort(SledTransactionServerError::OwnedBinaryError) => (
                atoms::abort(),
                Some(String::from(
                    "failed to allocate OwnedBinary for result value",
                )),
            ),
            TransactionError::Abort(SledTransactionServerError::UserAbort) => {
                (atoms::abort(), None)
            }
            TransactionError::Storage(err) => {
                (atoms::storage(), Some(format!("sled::result::{:?}", err)))
            }
        });
    (atoms::sled_transaction_complete(), result).encode(env)
}

pub fn transaction_close(tx_tree: SledTransactionalTree) -> SledTransactionalTreeSenderResult {
    tx_tree.send(SledTransactionalTreeRequest::new(move |_, _| {
        Ok(SledTransactionalTreeAction::Close)
    }))
}

pub fn transaction_abort(tx_tree: SledTransactionalTree) -> SledTransactionalTreeSenderResult {
    tx_tree.send(SledTransactionalTreeRequest::new(move |_, _| {
        Ok(SledTransactionalTreeAction::Abort)
    }))
}

pub fn transaction_insert(
    caller: LocalPid,
    tx_tree: SledTransactionalTree,
    k: Binary,
    v: Binary,
) -> SledTransactionalTreeSenderResult {
    let kv_cell = Arc::new((IVec::from(&k[..]), IVec::from(&v[..])));

    tx_tree.send(SledTransactionalTreeRequest::new(move |env, tx_tree| {
        let (k_ivec, v_ivec) = kv_cell.as_ref();
        let result = match tx_tree.insert(k_ivec, v_ivec)? {
            Some(v) => Some(try_ivec_to_binary(env, v)?),
            None => None,
        };
        send(env, &caller, atoms::sled_transaction_reply(), result);
        Ok(SledTransactionalTreeAction::Continue)
    }))
}

fn try_ivec_to_binary(
    env: Env,
    v: IVec,
) -> Result<Binary, ConflictableTransactionError<SledTransactionServerError>> {
    match OwnedBinary::new(v.len()) {
        Some(mut owned_binary) => {
            owned_binary.as_mut_slice().copy_from_slice(&v);
            Ok(owned_binary.release(env))
        }
        None => abort(SledTransactionServerError::OwnedBinaryError),
    }
}

fn send<T>(env: Env, caller: &LocalPid, tag: Atom, value: T)
where
    T: Encoder,
{
    env.send(&caller, (tag, value).encode(env))
}
