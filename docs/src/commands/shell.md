# Shell REPL: shell

Interactive REPL (Read-Eval-Print Loop) for exploratory and interactive use. The shell maintains a single persistent connection to the device for the duration of the session, avoiding the startup overhead of reconnecting for every command. Commands are the same as in non-interactive mode; the connection flags (`--host`, `--serial`, `--ble`) are specified once when launching the shell.

```bash
# Start an interactive shell (connects to default TCP host)
mttctl shell

# Start an interactive shell connected to a serial device
mttctl --serial /dev/ttyUSB0 shell
```

## Features

- Command history persisted to `~/.local/share/mttctl/history` across sessions
- Tab completion for all commands, subcommands, and flags (powered by `rustyline`)
- Single device connection reused for the entire session
- `help` prints available commands
- `exit` or Ctrl+D to quit

## Example Session

```
mttctl> nodes
  ID          Name    Battery  SNR    Hops  Last Heard
  !04e1c43b   Pedro   85%      8.5    0     just now
  !a1b2c3d4   Maria   72%      6.0    1     2m ago

mttctl> send "hello from shell"
ok Message sent.

mttctl> ping --to Maria
-> Pinging !a1b2c3d4 (Maria) (packet id: 7f3a1b2c)...
ok ACK from !a1b2c3d4 (Maria) in 1.8s

mttctl> exit
Goodbye.
```
