// Tests for task B01: simple fungible token (UDT).
// See tasks/B01-simple-udt/spec.md for what each case checks.

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

struct Setup {
    context: Context,
    owner: Script,
    alice: Script,
    udt: Script,
    other_udt: Script,
}

fn setup_with_args(args: impl FnOnce(&Script) -> Bytes) -> Setup {
    let mut context = Context::default();
    let bin = Loader::default().load_binary("simple-udt");
    let udt_code = context.deploy_cell(bin);
    let always = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let owner = context
        .build_script(&always, Bytes::from_static(b"owner"))
        .expect("owner lock");
    let alice = context
        .build_script(&always, Bytes::from_static(b"alice"))
        .expect("alice lock");
    let udt = context
        .build_script(&udt_code, args(&owner))
        .expect("udt type");
    // Same code, different args: a different token.
    let other_udt = context
        .build_script(&udt_code, Bytes::from(vec![7u8; 32]))
        .expect("other udt type");
    Setup {
        context,
        owner,
        alice,
        udt,
        other_udt,
    }
}

fn setup() -> Setup {
    setup_with_args(|owner| owner.calc_script_hash().as_bytes())
}

fn amount(value: u128) -> Bytes {
    Bytes::from(value.to_le_bytes().to_vec())
}

fn cell(lock: &Script, type_: Option<&Script>) -> CellOutput {
    CellOutput::new_builder()
        .capacity(1000u64)
        .lock(lock.clone())
        .type_(type_.cloned().pack())
        .build()
}

fn input(s: &mut Setup, lock: &Script, type_: Option<&Script>, data: Bytes) -> CellInput {
    let out_point = s.context.create_cell(cell(lock, type_), data);
    CellInput::new_builder().previous_output(out_point).build()
}

fn build_tx(
    s: &mut Setup,
    inputs: Vec<CellInput>,
    outputs: Vec<(CellOutput, Bytes)>,
) -> TransactionView {
    let (outputs, data): (Vec<_>, Vec<_>) = outputs.into_iter().unzip();
    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(data.into_iter().map(|d| d.pack()))
        .build();
    s.context.complete_tx(tx)
}

fn verify(s: &Setup, tx: &TransactionView) -> Result<u64, String> {
    s.context
        .verify_tx(tx, MAX_CYCLES)
        .map_err(|e| e.to_string())
}

// Output capacity is kept below input capacity in every test.
fn out(lock: &Script, type_: Option<&Script>) -> CellOutput {
    CellOutput::new_builder()
        .capacity(100u64)
        .lock(lock.clone())
        .type_(type_.cloned().pack())
        .build()
}

// ---------- Positive ----------

#[test]
fn p1_transfer() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, Some(&udt), amount(100));
    let tx = build_tx(
        &mut s,
        vec![i],
        vec![
            (out(&alice, Some(&udt)), amount(60)),
            (out(&alice, Some(&udt)), amount(40)),
        ],
    );
    verify(&s, &tx).expect("P1: transfer must pass");
}

#[test]
fn p2_burn() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, Some(&udt), amount(100));
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(30))]);
    verify(&s, &tx).expect("P2: burning part of the balance must pass");
}

#[test]
fn p3_owner_mint() {
    let mut s = setup();
    let (owner, alice, udt) = (s.owner.clone(), s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &owner, None, Bytes::new());
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(1000))]);
    verify(&s, &tx).expect("P3: owner must be able to mint");
}

#[test]
fn p4_merge_with_other_token_present() {
    let mut s = setup();
    let (alice, udt, other) = (s.alice.clone(), s.udt.clone(), s.other_udt.clone());
    let i0 = input(&mut s, &alice, Some(&udt), amount(50));
    let i1 = input(&mut s, &alice, Some(&udt), amount(50));
    let i2 = input(&mut s, &alice, Some(&other), amount(5));
    let tx = build_tx(
        &mut s,
        vec![i0, i1, i2],
        vec![
            (out(&alice, Some(&udt)), amount(100)),
            (out(&alice, Some(&other)), amount(5)),
        ],
    );
    verify(&s, &tx).expect("P4: merge with another token present must pass");
}

// ---------- Attacks ----------

#[test]
fn a1_mint_without_owner() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, None, Bytes::new());
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(100))]);
    assert!(verify(&s, &tx).is_err(), "A1: mint without owner was accepted");
}

#[test]
fn a2_inflate_by_one() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, Some(&udt), amount(100));
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(101))]);
    assert!(verify(&s, &tx).is_err(), "A2: 100 -> 101 was accepted");
}

#[test]
fn a3_overflow_wraps() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, Some(&udt), amount(1));
    // u128::MAX + 2 wraps around to 1, which equals the input.
    let tx = build_tx(
        &mut s,
        vec![i],
        vec![
            (out(&alice, Some(&udt)), amount(u128::MAX)),
            (out(&alice, Some(&udt)), amount(2)),
        ],
    );
    assert!(verify(&s, &tx).is_err(), "A3: overflowing outputs were accepted");
}

#[test]
fn a4_other_token_counted() {
    let mut s = setup();
    let (alice, udt, other) = (s.alice.clone(), s.udt.clone(), s.other_udt.clone());
    let i = input(&mut s, &alice, Some(&other), amount(1000));
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(1000))]);
    assert!(
        verify(&s, &tx).is_err(),
        "A4: another token's balance was counted as this token"
    );
}

#[test]
fn a5_owner_lock_on_output_only() {
    let mut s = setup();
    let (owner, alice, udt) = (s.owner.clone(), s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, None, Bytes::new());
    let tx = build_tx(&mut s, vec![i], vec![(out(&owner, Some(&udt)), amount(1000))]);
    assert!(
        verify(&s, &tx).is_err(),
        "A5: owner lock on an output was treated as owner mode"
    );
}

#[test]
fn a6_short_data() {
    let mut s = setup();
    let (alice, udt) = (s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &alice, Some(&udt), amount(100));
    let tx = build_tx(
        &mut s,
        vec![i],
        vec![
            (out(&alice, Some(&udt)), amount(100)),
            (out(&alice, Some(&udt)), Bytes::from(vec![0xff; 8])),
        ],
    );
    assert!(
        verify(&s, &tx).is_err(),
        "A6: token cell with 8-byte data was accepted"
    );
}

#[test]
fn a7_args_with_extra_byte() {
    let mut s = setup_with_args(|owner| {
        let mut args = owner.calc_script_hash().as_bytes().to_vec();
        args.push(0);
        Bytes::from(args)
    });
    let (owner, alice, udt) = (s.owner.clone(), s.alice.clone(), s.udt.clone());
    let i = input(&mut s, &owner, None, Bytes::new());
    let tx = build_tx(&mut s, vec![i], vec![(out(&alice, Some(&udt)), amount(1000))]);
    assert!(verify(&s, &tx).is_err(), "A7: 33-byte args were accepted");
}
