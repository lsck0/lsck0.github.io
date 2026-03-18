---
title: Hindley-Milner Type Inference
description: How ML-family languages infer types without annotations.
series: Programming Language Theory
series_order: 4
tags: cs, plt
publication: cs
project: plt
date: 2026-02-28
---

Hindley-Milner (HM) type inference lets the compiler figure out types for you. It powers OCaml, Haskell, and Rust's local inference.

## The Key Idea

Given an expression like

```rust
fn id(x) { x }
```

the algorithm asks: what constraints does the code impose on the types? Here, `x` is returned unchanged, so the input and output types must match. Result: `id : a -> a` for any type `a`.

## Algorithm W

The classic algorithm works in three steps:

1. **Generate**: assign fresh type variables to every subexpression
2. **Constrain**: collect equations from how expressions are used
3. **Unify**: solve the equations, substituting variables

## Unification

Given two type expressions, find a substitution that makes them equal:

$$\text{unify}(\alpha, \text{Int}) = [\alpha \mapsto \text{Int}]$$
$$\text{unify}(\alpha \to \beta, \text{Int} \to \text{Bool}) = [\alpha \mapsto \text{Int}, \beta \mapsto \text{Bool}]$$

Unification fails if a type variable would need to equal a type containing itself (the **occurs check**):

$$\text{unify}(\alpha, \alpha \to \text{Int}) = \text{error}$$

## Let-Polymorphism

The magic of HM is **let-polymorphism**: bindings introduced with `let` get a polymorphic type scheme, while lambda-bound variables stay monomorphic.

```rust
// id gets the polymorphic type: forall a. a -> a
let id = |x| x;

// both uses are valid: id is instantiated differently each time
let n: i32 = id(42);
let s: &str = id("hello");
```

This is decidable and runs in near-linear time for practical programs. No annotations needed.
