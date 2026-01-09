# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cosmodoro is a pomodoro timer applet for the COSMIC™ desktop environment, built with Rust using the libcosmic framework.

## Build System

This project uses [just](https://github.com/casey/just) as its command runner (not Make). All commands are defined in `justfile`.

### Common Commands

- `just` or `just build-release` - Build the application with release profile
- `just build-debug` - Build with debug profile
- `just run` - Build and run the application (uses `RUST_BACKTRACE=full`)
- `just check` - Run clippy linting with pedantic warnings
- `just clean` - Run `cargo clean`
- `just install` - Install the application (use `rootdir` and `prefix` vars to customize paths)

### Packaging Commands

- `just vendor` - Create vendored tarball of dependencies
- `just build-vendored` - Build using vendored dependencies (offline, frozen)
- `just rootdir=debian/cosmodoro prefix=/usr install` - Install to custom location

## Architecture

### Application Structure

The codebase follows a modular structure with clear separation of concerns:

- **main.rs**: Entry point that initializes i18n and launches the applet runtime
- **app.rs**: Core application logic implementing `cosmic::Application` trait
  - `AppModel`: Main application state (Core, config, timer state)
  - `Message`: Application messages enum for event handling
  - `TimerPhase`: Enum representing Idle, Work, ShortBreak, LongBreak states
  - `TimerState`: Tracks current phase, pause state, time remaining, completed sessions
  - `view()`: Renders the panel button with emoji and tooltip
  - `update()`: Message handler that returns Tasks
  - `subscription()`: Long-lived async tasks for timer ticks and config watching
- **config.rs**: Configuration persistence using `cosmic_config` with `CosmicConfigEntry` derive macro
- **i18n.rs**: Fluent-based localization system with `fl!()` macro for translations
- **sound.rs**: Sound effect playback for phase transitions

### COSMIC Applet Framework

This is a **COSMIC applet**, not a standalone application:
- Uses `cosmic::applet::run()` instead of `cosmic::app::run()`
- Simple single-button interface in the panel (no popup window)
- Panel button shows a single emoji representing the current state
- Tooltip displays time remaining and session count

### Key Dependencies

- **libcosmic**: COSMIC desktop framework (from git)
  - Features: `applet`, `applet-token`, `dbus-config`, `multi-window`, `tokio`, `wayland`, `winit`
- **tokio**: Async runtime
- **i18n-embed** + **i18n-embed-fl**: Fluent localization with `fluent-system` and `desktop-requester`
- **rust-embed**: Embeds i18n files into binary

### Configuration System

- Config struct derives `CosmicConfigEntry` with `#[version = 1]`
- Persisted automatically using cosmic-settings-daemon (via `dbus-config` feature)
- Watched for changes via `core.watch_config::<Config>()` subscription
- Loaded in `init()` with fallback to defaults on errors

### Internationalization

- Fluent files located in `i18n/[lang-code]/cosmodoro.ftl`
- English (`en`) is the fallback language (configured in `i18n.toml`)
- Access translations with `fl!("message-id")` macro
- Language auto-detected via `DesktopLanguageRequester` on startup
- New translations: copy `i18n/en` directory, rename to ISO 639-1 code, translate messages

## Development Notes

### Testing with libcosmic

To test against a local libcosmic clone, uncomment the `[patch]` section in `Cargo.toml`:

```toml
[patch.'https://github.com/pop-os/libcosmic']
libcosmic = { path = "../libcosmic" }
cosmic-config = { path = "../libcosmic/cosmic-config" }
cosmic-theme = { path = "../libcosmic/cosmic-theme" }
```

### Application ID

The app ID is `com.github.axelbdt.cosmodoro` (RDNN format), used for:
- Desktop file naming
- AppData/metainfo XML
- Icon file naming
- Configuration namespace

### UI Interaction Model

The applet uses a simple single-button interface:
- **Button Display**: Shows a single emoji representing the current timer phase
  - Idle: Empty/default icon
  - Work: 🍅 (tomato)
  - Break: ☕ (coffee)
  - Paused: ⏸️ (pause symbol)
- **Tooltip**: Displays time remaining (MM:SS) and session count (e.g., "[2/3]")
- **Button Click Actions**:
  - Idle → Start first work session
  - Running → Pause timer
  - Paused → Resume timer
  - Break complete → Start next work session (manual, no automatic detection)
- **Phase Transitions**:
  - Work complete → Automatic transition to short/long break
  - Break complete → Wait for user to manually start next work session via button click
- **Notifications**: Desktop notifications appear at phase transitions
- **Sounds**: Different sounds play when starting work vs starting break
