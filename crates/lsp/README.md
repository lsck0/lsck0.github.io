# blog-lsp

Language server for the blog's markdown content files. Indexes all post labels,
BibTeX entries, and post slugs from the `content/` directory and provides
real-time feedback while authoring.

## Features

| Feature           | Trigger      | Description                                                  |
| ----------------- | ------------ | ------------------------------------------------------------ |
| Autocomplete      | `[[`         | Suggests all labels (definitions, theorems, etc.) and post slugs |
| Autocomplete      | `[@`         | Suggests BibTeX keys from `content/references.bib`           |
| Go-to-definition  | `[[label]]`  | Jumps to the source file containing the labeled block        |
| Index rebuild     | On save      | Automatically re-indexes when any `.md` file is saved        |

## How it works

1. On startup, scans `content/posts/` recursively for `.md` files
2. Each file is parsed via the `ir` crate to extract frontmatter and labeled blocks
3. `content/references.bib` is parsed for citation keys
4. A content index maps labels to (kind, title, slug) and bib keys to display labels
5. Completions and definitions query this index
6. Saving any file triggers a full re-index

## Building

```bash
# development
cargo run --package lsp

# release binary (faster startup)
cargo build --release --package lsp
# binary at target/release/lsp
```

## Neovim setup

Add to your config (e.g. `~/.config/nvim/lua/plugins/blog-lsp.lua` or directly
in `init.lua`). Attaches only to markdown files inside the blog project:

```lua
-- blog content LSP: autocomplete for [[cross-refs]] and [@citations]
local blog_root = vim.fn.expand("~/code/lsck0.github.io")
local lsp_cmd = { blog_root .. "/target/release/lsp" }

-- fall back to cargo run if no release binary
if vim.fn.executable(lsp_cmd[1]) == 0 then
  lsp_cmd = { "cargo", "run", "--package", "lsp" }
end

vim.api.nvim_create_autocmd("FileType", {
  pattern = "markdown",
  callback = function(args)
    local file = vim.api.nvim_buf_get_name(args.buf)
    if not file:find(blog_root, 1, true) then return end

    vim.lsp.start({
      name = "blog-lsp",
      cmd = lsp_cmd,
      root_dir = blog_root,
      settings = {},
    })
  end,
})
```

With `lazy.nvim`:

```lua
return {
  dir = "~/code/lsck0.github.io",
  ft = "markdown",
  config = function()
    -- paste the autocmd block above here
  end,
}
```

## Other editors

Point any LSP client at `./target/release/lsp` (or `cargo run --package lsp`)
with the project root as the workspace directory. The server communicates over
stdin/stdout using the standard LSP protocol.
