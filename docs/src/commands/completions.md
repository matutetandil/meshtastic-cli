# Completions: completions

Generate shell completion scripts. Once installed, completions enable tab completion for all commands, subcommands, flags, and many argument values directly in your shell.

```bash
# Print completion script for the current shell to stdout
meshtastic-cli completions bash
meshtastic-cli completions zsh
meshtastic-cli completions fish
meshtastic-cli completions powershell
meshtastic-cli completions elvish
```

| Option | Description |
|---|---|
| `<SHELL>` | Target shell: `bash`, `zsh`, `fish`, `powershell`, `elvish` (required) |

## Installing Completions

### Bash

```bash
meshtastic-cli completions bash > ~/.local/share/bash-completion/completions/meshtastic-cli
```

### Zsh

```bash
meshtastic-cli completions zsh > ~/.zfunc/_meshtastic-cli
# Then add to ~/.zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

### Fish

```bash
meshtastic-cli completions fish > ~/.config/fish/completions/meshtastic-cli.fish
```
