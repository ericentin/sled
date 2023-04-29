use std::sync::Arc;

use crossbeam_channel::{bounded, Receiver};
use crossbeam_utils::atomic::AtomicCell;
use rustler::{spawn, Binary, Encoder, Env, LocalPid, NewBinary, Term, ThreadSpawner};
use sled::transaction::{abort, TransactionError};
use sled::{IVec, Tree};

use crate::types::{
    SledDbTree, SledTransactionServerError, SledTransactionalTree, SledTransactionalTreeAction,
    SledTransactionalTreeRequest, SledTransactionalTreeSenderResult,
};

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

    spawn::<ThreadSpawner, _>(env, move |thread_env: Env| {
        transaction_server(thread_env, &tree_cell.into_inner(), &r)
    });

    SledTransactionalTree::with_tree_and_sender(tree, s)
}

fn transaction_server<'a>(
    env: Env<'a>,
    tree: &Tree,
    r: &Receiver<SledTransactionalTreeRequest>,
) -> Term<'a> {
    let result = tree
        .transaction(move |tx_tree| loop {
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
        })
        .map_err(|tx_error| match tx_error {
            TransactionError::Abort(SledTransactionServerError::RecvError(err)) => (
                atoms::abort(),
                Some(format!("crossbeam-channel::err::{err:?}")),
            ),
            TransactionError::Abort(SledTransactionServerError::UserAbort) => {
                (atoms::abort(), None)
            }
            TransactionError::Storage(err) => {
                (atoms::storage(), Some(format!("sled::result::{err:?}")))
            }
        });
    (atoms::sled_transaction_complete(), result).encode(env)
}

pub fn transaction_close(tx_tree: &SledTransactionalTree) -> SledTransactionalTreeSenderResult {
    tx_tree.send(SledTransactionalTreeRequest::new(move |_, _| {
        Ok(SledTransactionalTreeAction::Close)
    }))
}

pub fn transaction_abort(tx_tree: &SledTransactionalTree) -> SledTransactionalTreeSenderResult {
    tx_tree.send(SledTransactionalTreeRequest::new(move |_, _| {
        Ok(SledTransactionalTreeAction::Abort)
    }))
}

pub fn transaction_insert(
    caller: LocalPid,
    tx_tree: &SledTransactionalTree,
    k: Binary,
    v: Binary,
) -> SledTransactionalTreeSenderResult {
    let kv_cell = Arc::new((IVec::from(&k[..]), IVec::from(&v[..])));

    tx_tree.send(SledTransactionalTreeRequest::new(move |env, tx_tree| {
        let (k_ivec, v_ivec) = kv_cell.as_ref();
        let result = tx_tree
            .insert(k_ivec, v_ivec)?
            .map(|v| ivec_to_binary(env, &v));
        env.send(
            &caller,
            (atoms::sled_transaction_reply(), result).encode(env),
        );
        Ok(SledTransactionalTreeAction::Continue)
    }))
}

fn ivec_to_binary<'a>(env: Env<'a>, v: &IVec) -> Binary<'a> {
    let mut new_binary = NewBinary::new(env, v.len());
    new_binary.copy_from_slice(v);
    Binary::from(new_binary)
}
