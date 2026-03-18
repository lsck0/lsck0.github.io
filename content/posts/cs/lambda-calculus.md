---
title: Lambda Calculus in a Nutshell
description: The smallest universal programming language, in three rules.
series: Programming Language Theory
series_order: 1
tags: cs, math, plt
publication: cs
project: plt
sources: https://plato.stanford.edu/entries/lambda-calculus/
date: 2026-03-05
---

The lambda calculus, introduced by Alonzo Church in the 1930s, is a formal system for expressing computation. Everything is a function.

## Syntax

There are only three kinds of terms:

1. **Variable**: $x$
2. **Abstraction**: $\lambda x.\, M$ (a function with parameter $x$ and body $M$)
3. **Application**: $M\; N$ (applying function $M$ to argument $N$)

That's it. No numbers, no strings, no loops. Everything else is encoded.

## Church Numerals

Natural numbers encoded as repeated application:

$$0 \equiv \lambda f.\, \lambda x.\, x$$
$$1 \equiv \lambda f.\, \lambda x.\, f\; x$$
$$2 \equiv \lambda f.\, \lambda x.\, f\; (f\; x)$$
$$n \equiv \lambda f.\, \lambda x.\, f^n\; x$$

The successor function adds one more application of $f$:

$$\text{SUCC} \equiv \lambda n.\, \lambda f.\, \lambda x.\, f\; (n\; f\; x)$$

## Boolean Logic

$$\text{TRUE} \equiv \lambda a.\, \lambda b.\, a$$
$$\text{FALSE} \equiv \lambda a.\, \lambda b.\, b$$
$$\text{AND} \equiv \lambda p.\, \lambda q.\, p\; q\; p$$

## The Y Combinator

Recursion without names:

$$Y \equiv \lambda f.\, (\lambda x.\, f\; (x\; x))\; (\lambda x.\, f\; (x\; x))$$

This fixed-point combinator satisfies $Y\; g = g\; (Y\; g)$ for any $g$.

## In Rust

A taste of lambda calculus style in Rust:

```rust
// Church encoding of booleans
fn church_true<A>(a: A, _b: A) -> A { a }
fn church_false<A>(_a: A, b: A) -> A { b }

// Church encoding of pairs
fn pair<A: Copy, B: Copy>(a: A, b: B) -> impl Fn(bool) -> (A, B) {
    move |select| if select { (a, b) } else { (a, b) }
}
```

Lambda calculus proves that a language needs almost nothing to be computationally universal.
