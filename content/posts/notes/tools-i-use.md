---
title: Tools I Use
description: My current development setup.
tags: notes, tools
---

A snapshot of what I'm using day to day. This changes, but slowly.

## Editor

Neovim with a minimal config. LSP for Rust and TypeScript. No file tree plugin; I use fuzzy finding.

## Terminal

Alacritty + tmux. Fast, simple, stays out of the way.

## Languages

- **Rust** for systems work, CLI tools, and this blog
- **TypeScript** when the browser is involved
- **Python** for quick scripts and data exploration

## Shell

```bash
# the essentials
alias ll='ls -la'
alias gs='git status'
alias gc='git commit'
alias gd='git diff'
```

Zsh with minimal plugins. History search and directory jumping are the only must-haves.

## Version Control

Git, obviously. Small commits, descriptive messages, rebase workflow.

## This Blog

Built with Leptos (Rust WASM framework), rendered client-side. Markdown content compiled into the binary at build time. Hosted on GitHub Pages.
