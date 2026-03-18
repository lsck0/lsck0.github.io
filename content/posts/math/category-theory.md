---
title: Category Theory Fundamentals
description: Categories, functors, natural transformations, and the language of modern mathematics.
tags: math, category-theory
date: 2026-03-14
---

**Category theory** provides a high-level language for mathematics, formalizing abstraction itself.

## Definition

A **category** $\mathcal{C}$ consists of:
- A class of **objects** $\text{Ob}(\mathcal{C})$
- For each pair of objects $(A, B)$, a set $\text{Hom}(A, B)$ of **morphisms**
- Composition: $\text{Hom}(B, C) \times \text{Hom}(A, B) \to \text{Hom}(A, C)$
- Identity: $\text{id}_A \in \text{Hom}(A, A)$

```definition Category {#def:category}
A category $\mathcal{C}$ consists of objects and morphisms satisfying:
1. Composition is associative
2. Identity morphisms behave as expected
```

## Examples

- **Set**: objects are sets, morphisms are functions
- **Grp**: objects are groups, morphisms are group homomorphisms
- **Top**: objects are topological spaces, morphisms are continuous maps
- **Vect$_K$**: objects are vector spaces over $K$, morphisms are linear maps

## Functors

A **functor** $F: \mathcal{C} \to \mathcal{D}$ maps:
- Objects: $F(A) \in \text{Ob}(\mathcal{D})$
- Morphisms: $F(f: A \to B) \to F(A) \to F(B)$

Functors preserve composition and identities.

```axiom Functor Composition {#ax:functor-compose}
F(g \circ f) = F(g) \circ F(f)
```

```axiom Identity Functor {#ax:identity-functor}
1_{\mathcal{C}}(A) = A \quad \text{and} \quad 1_{\mathcal{C}}(f) = f
```

## Natural transformations

Given functors $F, G: \mathcal{C} \to \mathcal{D}$, a **natural transformation** $\eta: F \Rightarrow G$ assigns to each object $A$ a morphism $\eta_A: F(A) \to G(A)$ such that for any $f: A \to B$:

$$G(f) \circ \eta_A = \eta_B \circ F(f)$$

## Limits and colimits

```definition Limit {#def:limit}
A limit of a diagram $D: \mathcal{J} \to \mathcal{C}$ is a universal cone to $D$.
```

```definition Colimit {#def:colimit}
A colimit of a diagram $D: \mathcal{J} \to \mathcal{C}$ is a universal cocone from $D$.
```

Examples:
- Products and coproducts (limits/colimits over discrete categories)
- Equalizers and coequalizers
- Pullbacks and pushouts
