mod server;

use crossbeam_channel::unbounded;
use crossbeam_utils::atomic::AtomicCell;
use rustler::{spawn, Binary, Env, LocalPid, ThreadSpawner};
use sled::IVec;

use crate::types::*;
use server::transaction_server;

pub fn transaction_new(env: Env, tree: SledDbTree) -> SledTransactionalTree {
    let (s, r) = unbounded();
    let tree_cell = AtomicCell::new(tree.clone());

    spawn::<ThreadSpawner, _>(env, move |thread_env: Env| {
        transaction_server(thread_env, &tree_cell.into_inner(), &r)
    });

    SledTransactionalTree::with_tree_and_sender(tree, s)
}

pub fn transaction_close(
    env: Env,
    tx_tree: SledTransactionalTree,
    req_ref: Binary,
) -> SledTransactionalTreeSenderResult {
    request(
        tx_tree,
        env.pid(),
        req_ref,
        SledTransactionalTreeCommand::Close,
    )
}

pub fn transaction_insert(
    env: Env,
    tx_tree: SledTransactionalTree,
    req_ref: Binary,
    k: Binary,
    v: Binary,
) -> SledTransactionalTreeSenderResult {
    let k_ivec = IVec::from(&k[..]);
    let v_ivec = IVec::from(&v[..]);
    request(
        tx_tree,
        env.pid(),
        req_ref,
        SledTransactionalTreeCommand::Insert(k_ivec, v_ivec),
    )
}

fn request(
    tx_tree: SledTransactionalTree,
    caller: LocalPid,
    req_ref: Binary,
    command: SledTransactionalTreeCommand,
) -> SledTransactionalTreeSenderResult {
    tx_tree.send((caller, Vec::from(&req_ref[..]), command))
}
