# ccswarm Herdr Plugin

Follow ccswarm runs and check its environment from [Herdr](https://herdr.dev).
The plugin needs Herdr 0.9.5 or later and ccswarm 0.11.0 or later as
`ccswarm` on the Herdr server's `PATH` (check with `ccswarm herdr --help`).

```bash
herdr plugin install nwiizo/ccswarm/herdr-plugin --ref v0.11.0
# or, from a checkout
herdr plugin link /path/to/ccswarm/herdr-plugin
```

| Action | Result |
| --- | --- |
| `ccswarm.run-board` | Split pane that follows the current run (`ccswarm tail`) |
| `ccswarm.runs` | Popup with recent runs (`ccswarm run list`) |
| `ccswarm.doctor` | Popup with `ccswarm doctor` |

Actions use the focused pane's directory as the repository. Each view waits
for Enter before closing, and Herdr shows one popup at a time. Bind actions
in your Herdr `config.toml`:

```toml
[[keys.command]]
key = "prefix+r"
type = "plugin_action"
command = "ccswarm.run-board"
description = "ccswarm run board"
```

Stage tabs, state reports, and notifications are part of ccswarm itself and
work without this plugin. See [Herdr Integration](../docs/HERDR_INTEGRATION.md).
