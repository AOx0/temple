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
