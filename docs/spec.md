# Pomodoro Timer - Cosmic Applet Specification

## Architecture

Single Rust binary implementing Cosmic applet protocol. No daemon, no CLI. Applet communicates with cosmic-panel via DBus, manages all timer logic internally.

## Core Flow

```
cosmic-panel loads applet
→ Applet registers with panel via cosmic::applet
→ Creates panel button widget
→ Runs timer loop in async task
→ Updates button state continuously
→ User controls via popup menu
→ Applet unloads when panel closes or user removes it
```

## Timer Cycle

**3 work sessions per cycle:**
- Work (25min) → Short Break (5min) → Work (25min) → Short Break (5min) → Work (25min) → Long Break (20min) → [repeat]

**Phase transitions:**
- Work completes → Break starts immediately (notification + sound)
- Break completes → Enter WAITING state (notification + sound)
- User activity detected → Next work starts automatically
- Manual start available via menu during WAITING

## State Machine

```
IDLE (0/3)
  ↓ [Start clicked]
WORK (1/3) - 25:00
  ↓ [timer expires]
SHORT_BREAK (1/3) - 5:00
  ↓ [timer expires]
WAITING_WORK (1/3)
  ↓ [activity detected OR Start clicked]
WORK (2/3) - 25:00
  ↓ [timer expires]
SHORT_BREAK (2/3) - 5:00
  ↓ [timer expires]
WAITING_WORK (2/3)
  ↓ [activity detected OR Start clicked]
WORK (3/3) - 25:00
  ↓ [timer expires]
LONG_BREAK (3/3) - 20:00
  ↓ [timer expires]
WAITING_WORK (0/3)  [counter resets]
  ↓ [activity detected OR Start clicked]
WORK (1/3) - 25:00  [cycle repeats]
```

**Pause behavior:**
- Pause available during WORK and BREAK phases only
- Paused timer shows ⏸️ prefix
- Not available during WAITING or IDLE

## Applet UI

### Button Widget

Button renders current state in panel with icon + text.

**Icons:**
- `idle` - Gray circle - IDLE or PAUSED
- `work` - Red tomato - WORK
- `break` - Green coffee cup - SHORT_BREAK and LONG_BREAK
- `waiting` - Yellow hourglass - WAITING_WORK

Use cosmic::widget::icon with system icon names or embedded SVG.

**Button text format:**
```
IDLE:           "Pomodoro"
WORK:           "🍅 25:00 [1/3]"
SHORT_BREAK:    "☕ 5:00 [1/3]"
LONG_BREAK:     "🌴 20:00 [3/3]"
WAITING:        "⏳ Ready [1/3]"
PAUSED:         "⏸️ 25:00 [1/3]"
```

Format: `[emoji] [MM:SS] [completed/total]`
- Updates every second via Message::Tick
- Use cosmic::widget::button with text and icon

### Popup Menu

Menu appears on button click. Built with cosmic::widget menu system.

```
Start/Resume
Pause
Skip Phase
Reset
─────────────
Quit
```

**Menu item states:**

| State | Start/Resume | Pause | Skip Phase | Reset |
|-------|-------------|-------|------------|-------|
| IDLE | Enabled | Disabled | Disabled | Disabled |
| WORK running | Disabled | Enabled | Enabled | Enabled |
| WORK paused | Enabled | Disabled | Enabled | Enabled |
| BREAK running | Disabled | Enabled | Enabled | Enabled |
| BREAK paused | Enabled | Disabled | Enabled | Enabled |
| WAITING | Enabled | Disabled | Enabled | Enabled |

**Actions:**
- **Start/Resume**: IDLE → Start first work; WAITING → Start next work; PAUSED → Resume
- **Pause**: WORK/BREAK running → Pause timer
- **Skip Phase**: WORK → Skip to break; BREAK → Skip to WAITING; WAITING → Start next work
- **Reset**: Any state → IDLE (0/3)
- **Quit**: Exit applet (no state persistence)

## Notifications

Use libnotify-rs or cosmic notification system:

```rust
notify_rust::Notification::new()
    .summary("Pomodoro Timer")
    .body(message)
    .icon("appointment-soon")
    .show()
```

**Notification triggers:**

| Event | Message |
|-------|---------|
| Work complete (→ short break) | "Work complete! Time for a short break." |
| Work complete (→ long break) | "Work complete! Time for a long break." |
| Break complete (→ waiting) | "Break over! Move your mouse when ready." |
| Timer paused | "Timer paused" |
| Timer resumed | "Timer resumed" |

## Sound

Embedded WAV files using include_bytes!:

**work.wav** (break → work transition):
- 800Hz sine wave
- 0.3 seconds duration
- 16-bit PCM, 44.1kHz sample rate

**break.wav** (work → break transition):
- 600Hz sine wave
- 0.5 seconds duration
- 16-bit PCM, 44.1kHz sample rate

Playback using rodio crate:
```rust
let (_stream, handle) = rodio::OutputStream::try_default()?;
let sink = rodio::Sink::try_new(&handle)?;
sink.append(rodio::Decoder::new(Cursor::new(WORK_WAV))?);
sink.sleep_until_end();
```

Silent fail if audio output unavailable.

## Activity Detection

Use X11 or Wayland idle detection during WAITING state.

**X11 (via x11rb):**
- Query XIdle extension
- Poll every 500ms

**Wayland (via cosmic-idle-inhibit or ext-idle-notify):**
- Subscribe to idle state changes
- Fallback to 500ms polling if unavailable

**Logic:**
- Track previous idle time
- Activity detected if: `currentIdle < previousIdle` OR `currentIdle < 1000ms`
- On activity: transition to next work phase
- If detection unavailable: disable auto-start, require manual Start

## Configuration

TOML file at `~/.config/pomodoro-applet/config.toml`:

```toml
work_minutes = 25
short_break_minutes = 5
long_break_minutes = 20
work_sessions_per_cycle = 3
```

Load on startup using serde + toml crate. Missing file or invalid values → defaults above.

No hot-reload: changes require applet reload.

## Edge Cases

**Applet removed from panel:** All state lost, no recovery.

**Activity detection fails:** User must manually click Start in WAITING state.

**Sound playback fails:** Silent, continue timer normally.

**Notification fails:** Silent, continue timer normally.

**Multiple applet instances:** Each maintains independent state. No synchronization.

## Non-Features

- No CLI interface
- No daemon/client architecture
- No state persistence across applet reloads
- No pause during WAITING/IDLE
- No custom notification sounds via config (embedded only)
- No statistics or history tracking
- No cross-desktop compatibility (Cosmic only)
