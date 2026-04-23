# cosmic-proxy-status

A [COSMIC desktop](https://github.com/pop-os/cosmic-de) panel applet that monitors an HTTP proxy connection. It sits in the panel next to the volume and network icons, showing live status at a glance and letting you start/stop polling and adjust settings from a popup.

![Panel icon with popup showing Connected, 42ms latency, HTTP 200](https://via.placeholder.com/600x300?text=screenshot)

## Features

- **Panel icon** — changes between connected, checking, and disconnected states
- **Popup** — shows status, latency (ms), HTTP status code, last checked timestamp, and error messages
- **Start / Stop monitoring** — toggle periodic polling from the popup
- **Configurable** — proxy URL, check URL, and poll interval (1–300 seconds), persisted via `cosmic_config`
- **10-second timeout** — checks never hang; network errors are reported cleanly

## Requirements

- COSMIC desktop (Pop!\_OS 24.04 or later)
- Rust 2024 edition (`rustup` recommended)
- `libxkbcommon` runtime library (`libxkbcommon.so.0`)

## Building

```bash
git clone https://github.com/spencerkittleson/cosmic-proxy-status
cd cosmic-proxy-status
cargo build
```

> **Note:** The `.cargo/config.toml` sets `PKG_CONFIG_PATH` and linker flags for `libxkbcommon` using the Flatpak SDK. If you have `libxkbcommon-dev` installed system-wide (`sudo apt install libxkbcommon-dev`) you can remove `.cargo/config.toml` and build normally.

## Installing

```bash
# 1. Build
cargo build

# 2. Install binary
cp target/debug/cosmic-proxy-status ~/.local/bin/

# 3. Install desktop entry (tells COSMIC panel this is an applet)
mkdir -p ~/.local/share/applications
cp data/dev.cosmic.ProxyStatus.desktop ~/.local/share/applications/

# 4. Install icon
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
cp data/icons/hicolor/scalable/apps/dev.cosmic.ProxyStatus-symbolic.svg \
   ~/.local/share/icons/hicolor/scalable/apps/

# 5. Add to panel (edit right-side plugins list)
# Open ~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings
# and add "dev.cosmic.ProxyStatus" to the right-side array.

# 6. Restart the panel
killall cosmic-panel
```

## Configuration

Settings are saved automatically when you click **Save** in the popup.

| Setting | Default | Description |
|---|---|---|
| Proxy URL | `http://192.168.8.204:3129` | The proxy to test |
| Check URL | `http://example.com` | The URL fetched through the proxy |
| Interval | `10` | Seconds between automatic checks (1–300) |

Config is stored at `~/.config/cosmic/dev.cosmic.ProxyStatus/`.

## Project Structure

```
src/
  main.rs      — entry point (cosmic::applet::run)
  app.rs       — applet logic, UI, messages, subscriptions
  checker.rs   — async HTTP check via reqwest
  config.rs    — AppConfig struct with cosmic_config persistence
data/
  dev.cosmic.ProxyStatus.desktop           — panel applet registration
  icons/hicolor/scalable/apps/
    dev.cosmic.ProxyStatus-symbolic.svg    — panel icon
```

## License

MPL-2.0
