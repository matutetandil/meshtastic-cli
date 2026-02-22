# Config File: config-file

Manage a persistent configuration file stored at `~/.config/mttctl/config.toml`. Values set here are applied automatically on every invocation, so you do not have to repeat connection options or other defaults on every command. Command-line flags always override config file values.

```bash
# Show current config file contents
mttctl config-file show

# Print the path to the config file
mttctl config-file path

# Set a persistent default value
mttctl config-file set host 192.168.1.100
mttctl config-file set port 4403
mttctl config-file set serial /dev/ttyUSB0

# Remove a previously set value (revert to built-in default)
mttctl config-file unset host
mttctl config-file unset serial
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

`~/.config/mttctl/config.toml`:

```toml
host = "192.168.1.100"
port = 4403
```
