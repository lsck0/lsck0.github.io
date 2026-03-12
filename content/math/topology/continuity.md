---
title: What Continuity Really Means
date: 2026-02-18
tags: math, topology, analysis
publication: math
description: From epsilon-delta to open sets.
---

The epsilon-delta definition of continuity is familiar but hides the geometric intuition. Topology reveals what's really going on.

## The Epsilon-Delta Definition

A function $f: \mathbb{R} \to \mathbb{R}$ is **continuous at** $a$ if:

$$\forall \varepsilon > 0,\; \exists \delta > 0 : |x - a| < \delta \implies |f(x) - f(a)| < \varepsilon$$

## The Topological Definition

A function $f: X \to Y$ between topological spaces is **continuous** if the preimage of every open set is open:

$$V \subseteq Y \text{ open} \implies f^{-1}(V) \subseteq X \text{ open}$$

This is the same idea, stated without distances. The epsilon-balls *are* the open sets in $\mathbb{R}$.

## Why This Matters

The topological definition works in spaces where distance makes no sense:

- Function spaces
- Quotient spaces
- Spaces of shapes (up to deformation)

## A Picture

```tikz
\begin{tikzpicture}[scale=1.2]
  % Domain
  \draw (0,0) ellipse (1.5 and 1);
  \node at (0, -1.4) {$X$};
  \fill (0.3, 0.1) circle (1.5pt) node[below] {$a$};
  \draw[blue, thick] (0.3, 0.1) circle (0.5);
  \node[blue] at (0.3, 0.75) {$f^{-1}(V)$};

  % Arrow
  \draw[->, thick] (2, 0) -- (3, 0) node[midway, above] {$f$};

  % Codomain
  \draw (4.5, 0) ellipse (1.5 and 1);
  \node at (4.5, -1.4) {$Y$};
  \fill (4.7, 0.1) circle (1.5pt) node[below] {$f(a)$};
  \draw[red, thick] (4.7, 0.1) circle (0.4);
  \node[red] at (4.7, 0.65) {$V$};
\end{tikzpicture}
```

Continuity means: the preimage of a small neighborhood around $f(a)$ is a neighborhood around $a$.
