# Completions: completions

Generate shell completion scripts. Once installed, completions enable tab completion for all commands, subcommands, flags, and many argument values directly in your shell.

```bash
# Print completion script for the current shell to stdout
mttctl completions bash
mttctl completions zsh
mttctl completions fish
mttctl completions powershell
mttctl completions elvish
```

| Option | Description |
|---|---|
| `<SHELL>` | Target shell: `bash`, `zsh`, `fish`, `powershell`, `elvish` (required) |

## Installing Completions

### Bash

```bash
mttctl completions bash > ~/.local/share/bash-completion/completions/mttctl
```

### Zsh

```bash
mttctl completions zsh > ~/.zfunc/_mttctl
# Then add to ~/.zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

### Fish

```bash
mttctl completions fish > ~/.config/fish/completions/mttctl.fish
```
