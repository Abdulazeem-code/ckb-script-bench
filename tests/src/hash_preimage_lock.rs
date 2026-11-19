// Tests for task A01: hash-preimage lock.
// See tasks/A01-hash-preimage-lock/spec.md for what each case checks.

use crate::Loader;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_hash::blake2b_256;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_testtool::context::Context;

const MAX_CYCLES: u64 = 10_000_000;
const PREIMAGE: &[u8] = b"ckb-script-bench secret preimage";

struct Setup {
    context: Context,
    lock: Script,
    other_lock: Script,
}

fn setup(args: Bytes) -> Setup {
    let mut context = Context::default();
    let bin = Loader::default().load_binary("hash-preimage-lock");
    let out_point = context.deploy_cell(bin);
    let lock = context.build_script(&out_point, args).expect("script");
    let always = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let other_lock = context
        .build_script(&always, Bytes::new())
        .expect("always success script");
    Setup {
        context,
        lock,
        other_lock,
    }
}

fn hash_args() -> Bytes {
    Bytes::from(blake2b_256(PREIMAGE).to_vec())
}

fn input(context: &mut Context, lock: &Script) -> CellInput {
    let out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64)
            .lock(lock.clone())
            .build(),
        Bytes::new(),
    );
    CellInput::new_builder().previous_output(out_point).build()
}

fn lock_witness(lock: &[u8]) -> Bytes {
    WitnessArgs::new_builder()
        .lock(Some(Bytes::from(lock.to_vec())).pack())
        .build()
        .as_bytes()
}

fn empty_witness() -> Bytes {
    WitnessArgs::new_builder().build().as_bytes()
}

fn build_tx(s: &mut Setup, inputs: Vec<CellInput>, witnesses: Vec<Bytes>) -> TransactionView {
    let output = CellOutput::new_builder()
        .capacity(500u64)
        .lock(s.other_lock.clone())
        .build();
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .output(output)
        .output_data(Bytes::new().pack())
        .witnesses(witnesses.into_iter().map(|w| w.pack()))
        .build();
    s.context.complete_tx(tx)
}

fn verify(s: &Setup, tx: &TransactionView) -> Result<u64, String> {
    s.context
        .verify_tx(tx, MAX_CYCLES)
        .map_err(|e| e.to_string())
}

// ---------- Positive ----------

#[test]
fn p1_correct_preimage() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i], vec![lock_witness(PREIMAGE)]);
    verify(&s, &tx).expect("P1: correct preimage must unlock");
}

#[test]
fn p2_two_inputs_one_witness() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i0 = input(&mut s.context, &lock);
    let i1 = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i0, i1], vec![lock_witness(PREIMAGE)]);
    verify(&s, &tx).expect("P2: one witness unlocks the whole group");
}

#[test]
fn p3_lock_input_at_index_1() {
    let mut s = setup(hash_args());
    let (lock, other) = (s.lock.clone(), s.other_lock.clone());
    let i0 = input(&mut s.context, &other);
    let i1 = input(&mut s.context, &lock);
    let tx = build_tx(
        &mut s,
        vec![i0, i1],
        vec![empty_witness(), lock_witness(PREIMAGE)],
    );
    verify(&s, &tx).expect("P3: witness at the group's input index must be used");
}

// ---------- Attacks ----------

#[test]
fn a1_wrong_preimage() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i], vec![lock_witness(b"not the secret")]);
    assert!(verify(&s, &tx).is_err(), "A1: wrong preimage was accepted");
}

#[test]
fn a2_no_witness() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i], vec![]);
    assert!(verify(&s, &tx).is_err(), "A2: missing witness was accepted");
}

#[test]
fn a3_preimage_in_input_type() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let witness = WitnessArgs::new_builder()
        .input_type(Some(Bytes::from(PREIMAGE.to_vec())).pack())
        .build()
        .as_bytes();
    let tx = build_tx(&mut s, vec![i], vec![witness]);
    assert!(
        verify(&s, &tx).is_err(),
        "A3: preimage in input_type was accepted"
    );
}

#[test]
fn a4_hash_instead_of_preimage() {
    let mut s = setup(hash_args());
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i], vec![lock_witness(&blake2b_256(PREIMAGE))]);
    assert!(
        verify(&s, &tx).is_err(),
        "A4: submitting the hash itself was accepted"
    );
}

#[test]
fn a5_args_with_extra_byte() {
    let mut args = blake2b_256(PREIMAGE).to_vec();
    args.push(0);
    let mut s = setup(Bytes::from(args));
    let lock = s.lock.clone();
    let i = input(&mut s.context, &lock);
    let tx = build_tx(&mut s, vec![i], vec![lock_witness(PREIMAGE)]);
    assert!(verify(&s, &tx).is_err(), "A5: 33-byte args were accepted");
}

#[test]
fn a6_witness_at_wrong_index() {
    let mut s = setup(hash_args());
    let (lock, other) = (s.lock.clone(), s.other_lock.clone());
    let i0 = input(&mut s.context, &other);
    let i1 = input(&mut s.context, &lock);
    // Valid preimage sits at witness 0, but the lock's group starts at input 1.
    let tx = build_tx(
        &mut s,
        vec![i0, i1],
        vec![lock_witness(PREIMAGE), empty_witness()],
    );
    assert!(
        verify(&s, &tx).is_err(),
        "A6: witness read from the wrong index was accepted"
    );
}
