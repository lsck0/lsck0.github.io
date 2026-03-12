---
title: Groups, Briefly
date: 2026-03-06
tags: math, algebra
publication: math
description: What makes a group a group, with commutative diagrams.
---

A **group** $(G, \cdot)$ is a set $G$ with a binary operation $\cdot$ satisfying:

1. **Closure**: $a \cdot b \in G$ for all $a, b \in G$
2. **Associativity**: $(a \cdot b) \cdot c = a \cdot (b \cdot c)$
3. **Identity**: there exists $e \in G$ such that $e \cdot a = a \cdot e = a$
4. **Inverse**: for each $a$, there exists $a^{-1}$ with $a \cdot a^{-1} = e$

## Homomorphisms

A **group homomorphism** $\varphi: G \to H$ preserves structure:

$$\varphi(a \cdot_G b) = \varphi(a) \cdot_H \varphi(b)$$

The first isomorphism theorem: if $\varphi: G \to H$ is a homomorphism, then

$$G / \ker(\varphi) \cong \text{im}(\varphi)$$

## A Commutative Diagram

The universal property of the quotient group:

```tikzcd
\begin{tikzcd}
  G \arrow[r, "\varphi"] \arrow[d, "\pi"'] & H \\
  G/\ker(\varphi) \arrow[ur, "\bar{\varphi}"', dashed] &
\end{tikzcd}
```

Here $\pi$ is the canonical projection and $\bar{\varphi}$ is the induced isomorphism.

## Examples

- $(\mathbb{Z}, +)$: integers under addition
- $(\mathbb{Z}/n\mathbb{Z}, +)$: integers modulo $n$
- $(S_n, \circ)$: symmetric group on $n$ elements
- $(GL_n(\mathbb{R}), \cdot)$: invertible $n \times n$ matrices

The study of groups is really the study of symmetry.
