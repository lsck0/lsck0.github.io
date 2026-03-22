---
title: Fixed Point Theorems
description: When functions must have a point that maps to itself.
tags: math, analysis
publication: math
sources: https://en.wikipedia.org/wiki/Banach_fixed-point_theorem, https://en.wikipedia.org/wiki/Brouwer_fixed-point_theorem
---

A **fixed point** of a function $f$ is a value $x$ where $f(x) = x$. Several deep theorems guarantee their existence.

## Banach Fixed Point Theorem

If $(X, d)$ is a complete metric space and $f: X \to X$ is a **contraction** (there exists $0 \le q < 1$ such that $d(f(x), f(y)) \le q \cdot d(x, y)$ for all $x, y$), then $f$ has a unique fixed point.

Moreover, for any starting point $x_0$, the iteration $x_{n+1} = f(x_n)$ converges to it:

$$d(x_n, x^*) \le \frac{q^n}{1 - q} d(x_0, x_1)$$

## Brouwer Fixed Point Theorem

Every continuous function $f: B^n \to B^n$ from the closed unit ball to itself has a fixed point.

This is non-constructive: it tells you a fixed point exists but gives no method to find it.

## A Geometric View

```tikz
\begin{tikzpicture}
  \draw[->] (-0.5,0) -- (4.5,0) node[right] {$x$};
  \draw[->] (0,-0.5) -- (0,4.5) node[above] {$y$};
  \draw[gray, dashed] (0,0) -- (4,4) node[right] {$y = x$};
  \draw[blue, thick, domain=0:4, samples=100] plot (\x, {0.5*\x + 1.2});
  \fill[red] (2.4, 2.4) circle (2pt) node[above right] {$x^*$};
  \node[blue] at (3.5, 3.3) {$f(x)$};
\end{tikzpicture}
```

The fixed point is where $f$ crosses the diagonal $y = x$.

## Application: Newton's Method

Newton's method for finding roots of $g(x) = 0$ defines the iteration

$$x_{n+1} = x_n - \frac{g(x_n)}{g'(x_n)}$$

Under suitable conditions, this is a contraction, and its fixed point satisfies $g(x^*) = 0$.

Fixed points also appear in computation theory: the Y combinator in [lambda calculus](/blog/cs/lambda-calculus) is a fixed-point operator that enables recursion without self-reference.
