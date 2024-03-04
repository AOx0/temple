# tree-sitter-temple

## Helix configuration

In `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "temple"
scope = "source.temple"
file-types = ["tpl", "temple"]
indent = { tab-width = 1, unit = "    " }
comment-token = "#"
injection-regex = "t(em)?ple?"

[language.auto-pairs]
'(' = ')'
'{' = '}'
'[' = ']'
'$' = '$'
'"' = '"'

[[grammar]]
name = "temple"
source.git = "https://github.com/AOx0/temple/"
source.rev = "tree-sitter"
```

Copy `bindings` as `~/.config/helix/runtime/bindings/temple`
Copy `queries` as `~/.config/helix/runtime/queries/temple`

As:

```bash
cp -r bindings ~/.config/helix/runtime/bindings/temple
cp -r queries ~/.config/helix/runtime/queries/temple
```
