# Xenomit Bot

The GPUI desktop controller for Xenomit. It opens on a sign-in page and
validates a Backpage session before showing the dashboard. Choose **Ruffle**
to launch the sibling `xenomit-ruffle-client` player, **Flash OCX** to launch
the sibling `xenomit-flash-client` player, or **Packet client** to connect
directly through the sibling `xenomit-packet-client` game API adapter

Build the player and controller from the `repos` directory:

```powershell
cargo build --manifest-path .\xenomit-ruffle-client\Cargo.toml --bin xenomit_ruffle_client
cargo run --manifest-path .\xenomit-bot\Cargo.toml
```

The controller looks for `xenomit_ruffle_client.exe` in the same directory and then
in `xenomit-ruffle-client\target\debug` or `xenomit-ruffle-client\target\release`. Set
`XENOMIT_RUFFLE_CLIENT_EXE` to use a different build.

On Windows the controller looks for `xenomit_flash_client.exe` beside its own
executable and then in `xenomit-flash-client\build`. On Linux it looks for
`xenomit_flash_client` beside its executable and then in
`xenomit-flash-client-linux`. Set `XENOMIT_FLASH_CLIENT_EXE` to use a different
build. The Linux host publishes native memory frames only.

Sign in with a Backpage username and password on the startup page.
The password is masked, cleared from the input after submission, and is never
written to disk. If Backpage requires a CAPTCHA, switch to **Existing session**
on the sign-in page and enter the server and `dosid`. Tab and Shift+Tab cycle
between inputs; Enter submits the form. The session is validated and
passed to the game process over stdin; it is never written to the module
settings file. The GUI saves module and safety settings in
`%APPDATA%\XenomitBot\modules.json`.

For Ruffle, choose **API backend** inside Settings to switch between native
shared-memory frames and the legacy reflection pipe while the game runs.
Switching stops the active module before reconnecting to the same process;
press Start to resume. The backend preference is independent of the client
choice and is preserved when using the packet client. Packet sessions have
one TCP backend and do not launch a game window.

The map is the default view. Open **Settings** in the toolbar to configure the
module, safety options, map zones, players, and API status.

The Settings account panel shows the latest game-client output while the
client is running. The full log is saved to
`%APPDATA%\XenomitBot\client.log` and can be opened from that panel.

The controller links `xenomit-behavior-engine` through the shared
`BehaviorRuntime` contract. **Start** runs its collector, NPC-killer, or
kill-and-collect planner on a worker thread. The bridge supplies actual map
entity IDs, world coordinates, NPC/player markers, and health values; the
runtime maps planner actions to the game's movement, attack, and collectable
calls. **Stop** stops movement and deselects the active NPC.

The default rules use the selected module and feature toggles. To supply the
engine's full configuration, add a `behavior_config` object to
`%APPDATA%\XenomitBot\modules.json`; it is passed through to
`BehaviorEngine` and validated on Start. The current planner supports
`kill_collect`, `npc_killer`, `collect`, and `palladium`. Galaxy gates and
Eternal Blacklight remain unavailable until the behavior engine implements
those modes.
