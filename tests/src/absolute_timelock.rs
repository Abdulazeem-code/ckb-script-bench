// Tests for task A02: absolute block-number timelock.
// See tasks/A02-absolute-timelock/spec.md for what each case checks.

use crate::Loader;
use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::*,
    prelude::*,
};
use ckb_testtool::context::Context;

const MAX_CYCLES: u64 = 10_000_000;
const N: u64 = 1000;

const RELATIVE_FLAG: u64 = 1 << 63;
const METRIC_EPOCH: u64 = 0b01 << 61;
const METRIC_TIMESTAMP: u64 = 0b10 << 61;

struct Setup {
    context: Context,
    lock: Script,
    other_lock: Script,
}

fn setup(args: Bytes) -> Setup {
    let mut context = Context::default();
    let bin = Loader::default().load_binary("absolute-timelock");
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

fn n_args() -> Bytes {
    Bytes::from(N.to_le_bytes().to_vec())
}

fn input(s: &mut Setup, since: u64) -> CellInput {
    let out_point = s.context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64)
            .lock(s.lock.clone())
            .build(),
        Bytes::new(),
    );
    CellInput::new_builder()
        .previous_output(out_point)
        .since(since)
        .build()
}

fn build_tx(s: &mut Setup, inputs: Vec<CellInput>) -> TransactionView {
    let output = CellOutput::new_builder()
        .capacity(500u64)
        .lock(s.other_lock.clone())
        .build();
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .output(output)
        .output_data(Bytes::new().pack())
        .build();
    s.context.complete_tx(tx)
}

fn verify(s: &Setup, tx: &TransactionView) -> Result<u64, String> {
    s.context
        .verify_tx(tx, MAX_CYCLES)
        .map_err(|e| e.to_string())
}

fn run_with_since(sinces: &[u64]) -> Result<u64, String> {
    let mut s = setup(n_args());
    let inputs = sinces.iter().map(|&v| input(&mut s, v)).collect();
    let tx = build_tx(&mut s, inputs);
    verify(&s, &tx)
}

// ---------- Positive ----------

#[test]
fn p1_exact_block() {
    run_with_since(&[N]).expect("P1: since == N must unlock");
}

#[test]
fn p2_later_block() {
    run_with_since(&[N + 500]).expect("P2: since > N must unlock");
}

#[test]
fn p3_two_valid_inputs() {
    run_with_since(&[N, N + 1]).expect("P3: two valid inputs must unlock");
}

// ---------- Attacks ----------

#[test]
fn a1_since_zero() {
    assert!(run_with_since(&[0]).is_err(), "A1: since = 0 was accepted");
}

#[test]
fn a2_one_block_early() {
    assert!(
        run_with_since(&[N - 1]).is_err(),
        "A2: since = N - 1 was accepted"
    );
}

#[test]
fn a3_relative_block() {
    assert!(
        run_with_since(&[RELATIVE_FLAG | N]).is_err(),
        "A3: relative block number was accepted"
    );
}

#[test]
fn a4_absolute_timestamp() {
    assert!(
        run_with_since(&[METRIC_TIMESTAMP | 1_900_000_000]).is_err(),
        "A4: absolute timestamp was accepted"
    );
}

#[test]
fn a5_absolute_epoch() {
    assert!(
        run_with_since(&[METRIC_EPOCH | 0x00ff_ffff]).is_err(),
        "A5: absolute epoch was accepted"
    );
}

#[test]
fn a6_second_input_unlocked() {
    assert!(
        run_with_since(&[N, 0]).is_err(),
        "A6: second input without timelock was accepted"
    );
}

#[test]
fn a7_args_with_extra_byte() {
    let mut args = N.to_le_bytes().to_vec();
    args.push(0);
    let mut s = setup(Bytes::from(args));
    let i = input(&mut s, N);
    let tx = build_tx(&mut s, vec![i]);
    assert!(verify(&s, &tx).is_err(), "A7: 9-byte args were accepted");
}
