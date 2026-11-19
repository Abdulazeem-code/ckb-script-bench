# Pilot run 01 (2026-11-19)

First check of the bench on real model output. Each model got the prompt
in a fresh chat with no follow-up messages and no extra CKB context.
Answers are saved unedited.

- Prompt template: [`prompts/baseline-v1.md`](../../prompts/baseline-v1.md)
- Rendered prompts: [`prompts/`](prompts/)
- Runner: `scripts/run_candidate.sh <crate> <answer-dir>`

## Results: full spec

The full spec names the pitfalls (exact args length, group-index
witness, check every input, `since` flag bits).

| Model | Task | Builds | Positive | Attacks rejected | Verdict |
|---|---|---|---|---|---|
| ChatGPT (web, default model) | A01 hash-preimage lock | yes | 3/3 | 6/6 | Correct |
| Gemini (web, default model) | A02 absolute timelock | yes | 3/3 | 7/7 | Correct |

Model versions were not recorded for this run.

### What this shows

With the full spec, both models produced correct, safe scripts. The full
spec lists every check the attacks target, so it measures whether a model
can follow a checklist, not whether it knows which checks matter.

## Results: brief prompts

[`A01-...-brief.md`](prompts/A01-hash-preimage-lock-brief.md) and
[`A02-...-brief.md`](prompts/A02-absolute-timelock-brief.md) give only
the interface (args and witness format), the way a developer would
actually ask. The safety checks are left to the model. Same tests.

| Model | Task | Builds | Positive | Attacks rejected | Verdict |
|---|---|---|---|---|---|
| ChatGPT (free tier) | A01 hash-preimage lock | yes | 3/3 | 6/6 | Correct |
| Gemini Pro | A02 absolute timelock | yes | 3/3 | 7/7 | Correct |

Untested gap: Gemini's A02 answer accepts `N = 0` in args (a cell that is
never actually locked). The brief didn't forbid it and no test covers it.

## Conclusion

Tier A tasks (one condition, one cell type) are too easy for current
models to show unsafe behaviour, even from a brief prompt. Both models
used `Source::GroupInput`, checked exact args length, looped over every
group input and masked the `since` flag bits without being told to.

This makes Tier A a useful **floor**: a model that fails Tier A is not
usable for CKB scripts at all. Finding unsafe behaviour needs Tier B
tasks: multiple cells, amounts and cross-cell rules, where mistakes like
unchecked overflow, counting the wrong cell source, or checking ownership
on outputs instead of inputs are possible.

## Results: B01 simple UDT (Tier B), brief prompt

Prompts use [`prompts/baseline-v2.md`](../../prompts/baseline-v2.md)
("on-chain script" instead of "lock script", since B01 is a type script):
[full spec](prompts/B01-simple-udt.md), [brief](prompts/B01-simple-udt-brief.md).

| Model | Task | Builds as written | Positive | Attacks rejected | Verdict |
|---|---|---|---|---|---|
| ChatGPT (free tier) | B01 simple UDT (brief) | **no** | – | – | **Compile error** |
| Gemini Pro | B01 simple UDT (brief) | **no** | – | – | **Compile error** |

The verdicts above are the official scores. To also check the logic, each
answer got the smallest edit that makes it compile, saved separately in
`*-brief-fixed/`:

| Model | Compile errors | Minimal fix | Verdict after fix |
|---|---|---|---|
| ChatGPT (free tier) | Calls `Script::hash()`, which doesn't exist (3 places); missing `prelude` import for `Byte32::as_slice()` | `.hash()` → `.calc_script_hash()`; add `use ckb_std::ckb_types::prelude::*;` | Correct: 4/4, 7/7 |
| Gemini Pro | `match` on `SysError` misses 5 variants added in ckb-std 1.1 | Add one `_ => Self::Encoding,` arm | Correct: 4/4, 7/7 |

Both models handled every safety rule without being told: checked
overflow (`checked_add`), counted only this token's cells, looked for the
owner lock on inputs only, and rejected data shorter than 16 bytes.

## Conclusion (updated)

Across 6 answers (2 models × 3 tasks, full spec and brief), the models
never produced an unsafe script. Their failures were all **API
knowledge**: methods that don't exist in ckb-std 1.1, missing trait
imports, and enum variants added in newer versions. Both Tier B answers
failed to compile as written.

This points to the measurable problem: models know *what* a safe CKB
script must check, but not the *current* ckb-std API. The next
experiment is the same prompts with CKB's official AI context added
(the `AGENTS.md` shipped with ckb-script-templates), to measure whether
it fixes compile failures.
