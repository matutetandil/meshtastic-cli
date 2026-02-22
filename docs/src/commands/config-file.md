# Config File: config-file

Manage a persistent configuration file stored at `~/.config/meshtastic-cli/config.toml`. Values set here are applied automatically on every invocation, so you do not have to repeat connection options or other defaults on every command. Command-line flags always override config file values.

```bash
# Show current config file contents
meshtastic-cli config-file show

# Print the path to the config file
meshtastic-cli config-file path

# Set a persistent default value
meshtastic-cli config-file set host 192.168.1.100
meshtastic-cli config-file set port 4403
meshtastic-cli config-file set serial /dev/ttyUSB0

# Remove a previously set value (revert to built-in default)
meshtastic-cli config-file unset host
meshtastic-cli config-file unset serial
```

## Subcommands

| Subcommand | Description |
|---|---|
| `show` | Print the current config file contents as TOML |
| `set <KEY> <VALUE>` | Set a persistent default value |
| `unset <KEY>` | Remove a key, reverting to the built-in default |
| `path` | Print the filesystem path of the config file |

## Available Keys

| Key | Description | Equivalent flag |
|---|---|---|
| `host` | Default TCP host | `--host` |
| `port` | Default TCP port | `--port` |
| `serial` | Default serial device path | `--serial` |

## Example Config File

`~/.config/meshtastic-cli/config.toml`:

```toml
host = "192.168.1.100"
port = 4403
```
