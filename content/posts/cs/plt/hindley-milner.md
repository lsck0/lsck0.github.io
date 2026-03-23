---
title: Hindley-Milner Type Inference
description: How ML-family languages infer types without annotations.
series: Programming Language Theory
series_order: 4
tags: cs, plt
publication: cs
project: plt
toc: true
---

Hindley-Milner (HM) type inference lets the compiler figure out types for you. It powers OCaml, Haskell, and Rust's local inference.

## The Key Idea

```definition Type Inference {#def:type-inference}
**Type inference** is the automatic deduction of the type of an expression, without requiring explicit type annotations from the programmer.
```

Given an expression like

```rust
fn id(x) { x }
```

the algorithm asks: what constraints does the code impose on the types? Here, `x` is returned unchanged, so the input and output types must match. Result: `id : a -> a` for any type `a`. This connects directly to the [[cs/type-systems#def:curry-howard]]: the inferred type corresponds to the proposition the program proves.

## Algorithm W

The classic algorithm works in three steps:

1. **Generate**: assign fresh type variables to every subexpression
2. **Constrain**: collect equations from how expressions are used
3. **Unify**: solve the equations, substituting variables

## Unification

```definition Unification {#def:unification}
Given two type expressions, **unification** finds a substitution that makes them equal, or fails if no such substitution exists.
```

$$\text{unify}(\alpha, \text{Int}) = [\alpha \mapsto \text{Int}]$$
$$\text{unify}(\alpha \to \beta, \text{Int} \to \text{Bool}) = [\alpha \mapsto \text{Int}, \beta \mapsto \text{Bool}]$$

Unification fails if a type variable would need to equal a type containing itself (the **occurs check**):

$$\text{unify}(\alpha, \alpha \to \text{Int}) = \text{error}$$

## Let-Polymorphism

```definition Let-Polymorphism {#def:let-polymorphism}
In HM, bindings introduced with `let` get a **polymorphic type scheme** (universally quantified over free type variables), while [[cs/lambda-calculus#def:lambda-term]]-bound variables stay monomorphic.
```

```rust
// id gets the polymorphic type: forall a. a -> a
let id = |x| x;

// both uses are valid: id is instantiated differently each time
let n: i32 = id(42);
let s: &str = id("hello");
```

This is decidable and runs in near-linear time for practical programs. No annotations needed.
