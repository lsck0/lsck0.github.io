---
title: Fields
description: Fields, field extensions, and the characteristic.
series: Abstract Algebra
series_order: 3
tags: math, algebra
publication: math
toc: true
sources: https://en.wikipedia.org/wiki/Field_(mathematics), https://en.wikipedia.org/wiki/Field_extension
date: 2026-01-25
---

Fields are the algebraic structures where division is possible. This treatment follows Lang[^lang] ch. V and Morandi[^morandi].

[^lang]: Serge Lang, *Algebra*, 3rd ed., Springer, 2002.
[^morandi]: Patrick Morandi, *Field and Galois Theory*, Springer, 1996.

## Fields

```definition Field {#def:field}
A **field** $(F, +, \cdot)$ is a [[def:commutative-ring]] in which every nonzero element has a multiplicative inverse. That is, for all $a \in F \setminus \{0\}$, there exists $a^{-1} \in F$ with $a \cdot a^{-1} = 1$.

Equivalently, $(F, +)$ is an [[def:abelian-group]], $(F \setminus \{0\}, \cdot)$ is an abelian group, and multiplication distributes over addition.
```

```example {#ex:fields}
| Field | Notation | Characteristic | Cardinality |
|---|---|---|---|
| Rationals | $\mathbb{Q}$ | $0$ | $\aleph_0$ |
| Reals | $\mathbb{R}$ | $0$ | $\mathfrak{c}$ |
| Complex numbers | $\mathbb{C}$ | $0$ | $\mathfrak{c}$ |
| Integers mod $p$ | $\mathbb{F}_p$ | $p$ | $p$ |
| $p$-element field | $\mathbb{F}_{p^n}$ | $p$ | $p^n$ |
```

```definition Characteristic | char {#def:characteristic}
The **characteristic** of a [[def:field]] $F$, denoted $\text{char}(F)$, is the smallest positive integer $n$ such that $\underbrace{1 + \cdots + 1}_{n} = 0$, or $0$ if no such $n$ exists.
```

```proposition {#prop:char-prime}
The characteristic of any [[def:field]] is either $0$ or a prime number.
```

```proof {#proof:char-prime}
If $\text{char}(F) = n = ab$ with $1 < a, b < n$, then $(\underbrace{1+\cdots+1}_a)(\underbrace{1+\cdots+1}_b) = 0$. Since $F$ is a field (hence an [[def:integral-domain]]), one of the factors is zero, contradicting minimality of $n$.
```

```definition Prime Subfield {#def:prime-subfield}
The **prime subfield** of a [[def:field]] $F$ is the smallest subfield of $F$. It is isomorphic to $\mathbb{Q}$ if $\text{char}(F) = 0$, or $\mathbb{F}_p$ if $\text{char}(F) = p$.
```

## Field Extensions

```definition Field Extension {#def:field-extension}
A **field extension** $L/K$ is a pair of [[def:field|fields]] $K \subseteq L$ where the inclusion is a [[def:ring-hom|ring homomorphism]]. We call $K$ the **base field** and $L$ the **extension field**.
```

```definition Degree of Extension {#def:extension-degree}
The **degree** of a [[def:field-extension]] $L/K$ is the dimension of $L$ as a [[def:vector-space]] over $K$:

$$[L : K] = \dim_K L$$

If $[L:K] < \infty$, the extension is **finite**.
```

```theorem Tower Law {#thm:tower-law}
If $K \subseteq L \subseteq M$ are fields, then

$$[M : K] = [M : L] \cdot [L : K]$$
```

```proof {#proof:tower-law}
Let $\{e_i\}_{i=1}^m$ be a basis of $M$ over $L$ and $\{f_j\}_{j=1}^n$ a basis of $L$ over $K$. We claim $\{e_i f_j\}$ is a basis of $M$ over $K$.

**Spanning**: Any $x \in M$ can be written $x = \sum_i a_i e_i$ with $a_i \in L$. Each $a_i = \sum_j b_{ij} f_j$ with $b_{ij} \in K$. So $x = \sum_{i,j} b_{ij} e_i f_j$.

**Independence**: If $\sum_{i,j} b_{ij} e_i f_j = 0$, then $\sum_i (\sum_j b_{ij} f_j) e_i = 0$. By independence of $\{e_i\}$ over $L$, $\sum_j b_{ij} f_j = 0$ for all $i$. By independence of $\{f_j\}$ over $K$, $b_{ij} = 0$ for all $i, j$.
```

```tikzcd
\begin{tikzcd}
  M \arrow[d, dash, "{[M:L]}"'] \\
  L \arrow[d, dash, "{[L:K]}"'] \\
  K
\end{tikzcd}
```

```definition Algebraic Element {#def:algebraic}
An element $\alpha \in L$ is **algebraic** over $K$ if it is a root of some nonzero polynomial $f \in K[x]$. The **minimal polynomial** of $\alpha$ over $K$ is the unique monic irreducible polynomial in $K[x]$ with $\alpha$ as a root.
```

```definition Transcendental Element {#def:transcendental}
An element $\alpha \in L$ is **transcendental** over $K$ if it is not [[def:algebraic]]. In this case, $K(\alpha) \cong K(x)$, the field of rational functions.
```

```theorem {#thm:simple-extension}
If $\alpha$ is [[def:algebraic]] over $K$ with minimal polynomial $m_\alpha$ of degree $n$, then

$$K(\alpha) \cong K[x]/(m_\alpha)$$

and $[K(\alpha) : K] = n$. A basis for $K(\alpha)$ over $K$ is $\{1, \alpha, \alpha^2, \ldots, \alpha^{n-1}\}$.
```

```example {#ex:field-extension}
The extension $\mathbb{Q}(\sqrt{2})/\mathbb{Q}$ has degree $2$, since $\sqrt{2}$ has minimal polynomial $x^2 - 2$ over $\mathbb{Q}$. By [[thm:simple-extension]], $\{1, \sqrt{2}\}$ is a basis.

A computation in this field:

$$\frac{1}{1 + \sqrt{2}} = \frac{1 - \sqrt{2}}{(1+\sqrt{2})(1-\sqrt{2})} = \frac{1 - \sqrt{2}}{-1} = \sqrt{2} - 1$$
```

## Splitting Fields

```definition Splitting Field {#def:splitting-field}
A **splitting field** for a polynomial $f \in K[x]$ is a [[def:field-extension]] $L/K$ such that $f$ factors completely into linear factors over $L$, and $L$ is generated over $K$ by the roots of $f$.
```

```theorem {#thm:splitting-exists}
For any polynomial $f \in K[x]$ of degree $n$, a [[def:splitting-field]] exists and is unique up to isomorphism. Its degree over $K$ divides $n!$.
```

```example Splitting Field of $x^3 - 2$ {#ex:splitting-cubic}
Over $\mathbb{Q}$, the polynomial $x^3 - 2$ has roots $\sqrt[3]{2}$, $\omega\sqrt[3]{2}$, $\omega^2\sqrt[3]{2}$ where $\omega = e^{2\pi i/3}$. The splitting field is $\mathbb{Q}(\sqrt[3]{2}, \omega)$ with $[\mathbb{Q}(\sqrt[3]{2}, \omega) : \mathbb{Q}] = 6$.

By the [[thm:tower-law]]:

$$[\mathbb{Q}(\sqrt[3]{2}, \omega) : \mathbb{Q}] = [\mathbb{Q}(\sqrt[3]{2}, \omega) : \mathbb{Q}(\sqrt[3]{2})] \cdot [\mathbb{Q}(\sqrt[3]{2}) : \mathbb{Q}] = 2 \cdot 3 = 6$$
```

```tip
The theory of splitting fields leads naturally to **Galois theory**, which establishes a bijection between intermediate fields of a Galois extension and subgroups of its automorphism group. See Stewart[^stewart] for an accessible introduction.

[^stewart]: Ian Stewart, *Galois Theory*, 4th ed., CRC Press, 2015.
```

## Finite Fields[^2]

[^2]: Finite fields are also called **Galois fields** after Évariste Galois. The notation $\mathbb{F}_q$ or $GF(q)$ is standard.

```theorem Classification of Finite Fields {#thm:finite-fields}
For every prime power $q = p^n$, there exists a unique (up to isomorphism) finite field $\mathbb{F}_q$ of order $q$. Moreover:
- Every finite field has order $p^n$ for some prime $p$ and integer $n \geq 1$
- $\mathbb{F}_q$ is the [[def:splitting-field]] of $x^q - x$ over $\mathbb{F}_p$
- The multiplicative group $\mathbb{F}_q^*$ is cyclic of order $q - 1$
```

```example Constructing $\mathbb{F}_4$ {#ex:f4}
We need an irreducible quadratic over $\mathbb{F}_2$. The polynomial $x^2 + x + 1$ is irreducible (check: $0^2 + 0 + 1 = 1 \neq 0$ and $1^2 + 1 + 1 = 1 \neq 0$). Then:

$$\mathbb{F}_4 = \mathbb{F}_2[x]/(x^2 + x + 1) = \{0, 1, \alpha, \alpha + 1\}$$

where $\alpha^2 = \alpha + 1$. The addition and multiplication tables:

| $+$ | $0$ | $1$ | $\alpha$ | $\alpha+1$ |
|---|---|---|---|---|
| $0$ | $0$ | $1$ | $\alpha$ | $\alpha+1$ |
| $1$ | $1$ | $0$ | $\alpha+1$ | $\alpha$ |
| $\alpha$ | $\alpha$ | $\alpha+1$ | $0$ | $1$ |
| $\alpha+1$ | $\alpha+1$ | $\alpha$ | $1$ | $0$ |
```

```info
Finite fields are central to coding theory (Reed-Solomon codes, BCH codes), cryptography (elliptic curve cryptography uses $\mathbb{F}_p$), and combinatorics (counting points on algebraic varieties over $\mathbb{F}_q$).
```
