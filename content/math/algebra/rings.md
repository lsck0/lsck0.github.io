---
title: From Groups to Rings
date: 2026-01-30
tags: math, algebra
publication: math
description: Adding a second operation to the mix.
---

A **ring** $(R, +, \cdot)$ is a set with two operations where:

- $(R, +)$ is an abelian group (commutative, with identity $0$ and inverses)
- $(R, \cdot)$ is a monoid (associative, with identity $1$)
- Multiplication distributes over addition: $a(b + c) = ab + ac$

## Examples

- $(\mathbb{Z}, +, \cdot)$ -- the integers
- $(\mathbb{Z}[x], +, \cdot)$ -- polynomials with integer coefficients
- $(M_n(\mathbb{R}), +, \cdot)$ -- $n \times n$ matrices (not commutative!)

## Ring Homomorphisms

A map $\varphi: R \to S$ is a ring homomorphism if it preserves both operations:

$$\varphi(a + b) = \varphi(a) + \varphi(b)$$
$$\varphi(a \cdot b) = \varphi(a) \cdot \varphi(b)$$
$$\varphi(1_R) = 1_S$$

## Ideals

An **ideal** $I \subseteq R$ is a subring that absorbs multiplication:

$$r \in R, \; a \in I \implies ra \in I \text{ and } ar \in I$$

The quotient $R/I$ is again a ring. This mirrors how normal subgroups yield quotient groups.

```tikzcd
\begin{tikzcd}
  R \arrow[r, "\varphi"] \arrow[d, "\pi"'] & S \\
  R/\ker(\varphi) \arrow[ur, "\bar{\varphi}"', dashed, "\cong" near start] &
\end{tikzcd}
```

The first isomorphism theorem holds for rings too.
