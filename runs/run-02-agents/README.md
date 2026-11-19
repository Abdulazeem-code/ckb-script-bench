# Run 02: with CKB's official AI guide

**Question:** does CKB's official AI guide fix the compile failures seen
in [pilot run 01](../pilot-01/README.md)?

In run 01, both models' answers to the B01 (simple UDT) brief prompt
failed to compile against ckb-std 1.1. Each failure was API knowledge:
a method that doesn't exist (`Script::hash()`), a missing trait import,
and a `match` missing enum variants added in ckb-std 1.1.

**Change from run 01:** the prompt starts with the full `AGENTS.md` guide
shipped with cryptape/ckb-script-templates, included verbatim. Everything
after it (environment, answer format, task text) is identical to run 01.

- Template: [`prompts/agents-v1.md`](../../prompts/agents-v1.md)
- Prompts: [B01 brief](prompts/B01-simple-udt-brief.md), [B01 full spec](prompts/B01-simple-udt.md)

Same rules as run 01: fresh chat, prompt pasted unchanged, no follow-up
messages, answer saved unedited. The unedited answer is the official
score.

## Results

| Model | Task | Prompt | Builds as written | Positive | Attacks rejected | Verdict |
|---|---|---|---|---|---|---|
| ChatGPT (free tier) | B01 | brief + guide | **yes** | 4/4 | 7/7 | **Correct** |
| Gemini Pro | B01 | brief + guide | **no** | – | – | **Compile error** |

ChatGPT's answer compiled with no warnings. In run 01 it called
`Script::hash()`, which doesn't exist. With the guide it used the
`high_level` helpers the guide points to (`load_script_hash`,
`load_cell_type_hash`, `load_cell_lock_hash`) and avoided the problem.
It also filtered cells by type hash over `Source::Input`/`Output`
instead of using the script group, which is a valid alternative and
passes the "other token" attack (A4).

Gemini Pro made the identical mistake as in run 01: a `match` on
`SysError` that covers only the variants from older ckb-std versions and
misses 5 variants added in ckb-std 1.1 (`WaitFailure`, `InvalidFd`,
`OtherEndClosed`, ...). With the same one-line fix as in run 01 (add a
`_ => Self::Encoding,` arm, saved in `gemini-pro/B01-simple-udt-brief-fixed/`)
it is correct: 4/4 positive, 7/7 attacks rejected.

## Comparison with run 01

| Model | B01 brief, no guide (run 01) | B01 brief + guide (run 02) |
|---|---|---|
| ChatGPT (free tier) | Compile error (`Script::hash()`) | **Correct** |
| Gemini Pro | Compile error (`SysError` match) | Compile error (same `SysError` match) |

**Summary:** the guide fixed one of the two compile failures. It helped
where the failure was *which API to use* (ChatGPT invented a method; the
guide's "use `high_level` first" steered it to real ones). It didn't
help where the failure was *which version of the API* (Gemini's
`SysError` match is valid for older ckb-std, and the guide never
mentions versions).

One answer per model per condition, so this is a signal, not a
measurement. The full bench repeats each prompt several times.

## What the guide doesn't cover

`AGENTS.md` tells agents to use ckb-std's `high_level` module and
`ckb-types`, but says nothing about which API version is current. The
`SysError` failure is exactly that gap: the model's code matches older
ckb-std versions. A short "ckb-std 1.x changes" note in the guide (for
example: `SysError` has more variants, so add a `_` arm) would target the
failure seen in both runs.
