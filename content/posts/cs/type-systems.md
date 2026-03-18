---
title: Types as Propositions
description: The Curry-Howard correspondence connects type theory and logic.
series: Programming Language Theory
series_order: 3
tags: cs, plt, math
publication: cs
project: plt
sources: https://en.wikipedia.org/wiki/Curry%E2%80%93Howard_correspondence
date: 2026-02-15
---

The **Curry-Howard correspondence** reveals that types are propositions, programs are proofs, and computation is proof simplification.

## The Dictionary

| Logic             | Type Theory          |
| ----------------- | -------------------- |
| Proposition       | Type                 |
| Proof             | Term (program)       |
| $A \land B$       | $A \times B$ (pair)  |
| $A \lor B$        | $A + B$ (sum)        |
| $A \implies B$    | $A \to B$ (function) |
| $\top$ (true)     | Unit type            |
| $\bot$ (false)    | Empty type           |
| $\forall x. P(x)$ | Dependent product    |

## Example: Conjunction

To prove $A \land B$, you need a proof of $A$ and a proof of $B$. In code, that's constructing a pair:

```rust
fn and_intro<A, B>(a: A, b: B) -> (A, B) {
    (a, b)
}

fn and_elim_left<A, B>(pair: (A, B)) -> A {
    pair.0
}
```

## Example: Implication

A proof of $A \implies B$ is a function that transforms a proof of $A$ into a proof of $B$:

```rust
fn modus_ponens<A, B>(implication: impl Fn(A) -> B, proof_a: A) -> B {
    implication(proof_a)
}
```

## The Punchline

Every well-typed program in a strongly normalizing language _is_ a proof of the proposition expressed by its type. The compiler is a proof checker.

This is not just a metaphor. Proof assistants like Coq, Agda, and Lean are built directly on this correspondence.
