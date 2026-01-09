// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::sound;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::Subscription;
use cosmic::prelude::*;
use cosmic::widget::*;
use futures_util::SinkExt;

/// Represents the current phase of the pomodoro timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPhase {
    Idle,
    Work,
    ShortBreak,
    LongBreak,
}

/// Represents the complete state of the timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerState {
    pub phase: TimerPhase,
    pub is_paused: bool,
    pub time_remaining_secs: u32,
    pub completed_sessions: u32,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            phase: TimerPhase::Idle,
            is_paused: false,
            time_remaining_secs: 0,
            completed_sessions: 0,
        }
    }
}

impl TimerState {
    /// Formats the remaining time as MM:SS
    pub fn format_time(&self) -> String {
        let minutes = self.time_remaining_secs / 60;
        let seconds = self.time_remaining_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Gets the emoji for the current phase
    pub fn phase_emoji(&self) -> &'static str {
        if self.is_paused {
            return "⏸️";
        }
        match self.phase {
            TimerPhase::Idle => "⏸️",
            TimerPhase::Work => "🍅",
            TimerPhase::ShortBreak => "☕",
            TimerPhase::LongBreak => "🌴",
        }
    }

    /// Gets the tooltip text showing time and session count
    pub fn tooltip_text(&self) -> String {
        if self.phase == TimerPhase::Idle {
            format!("Click to start {}", self.session_counter())
        } else {
            format!("{} {}", self.format_time(), self.session_counter())
        }
    }

    /// Gets the session counter display
    pub fn session_counter(&self) -> String {
        if self.phase == TimerPhase::Idle {
            String::new()
        } else {
            format!("[{}/3]", self.completed_sessions)
        }
    }
}

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
#[derive(Default)]
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Configuration data that persists between application runs.
    config: Config,
    /// The current timer state.
    timer_state: TimerState,
}

impl AppModel {
    /// Starts a work session
    fn start_work(&mut self) {
        self.timer_state.phase = TimerPhase::Work;
        self.timer_state.is_paused = false;
        self.timer_state.time_remaining_secs = self.config.work_minutes * 60;
        self.timer_state.completed_sessions += 1;
    }

    /// Starts a short break
    fn start_short_break(&mut self) {
        self.timer_state.phase = TimerPhase::ShortBreak;
        self.timer_state.is_paused = false;
        self.timer_state.time_remaining_secs = self.config.short_break_minutes * 60;
    }

    /// Starts a long break
    fn start_long_break(&mut self) {
        self.timer_state.phase = TimerPhase::LongBreak;
        self.timer_state.is_paused = false;
        self.timer_state.time_remaining_secs = self.config.long_break_minutes * 60;
    }

    /// Handles timer completion and transitions to next phase
    fn handle_timer_complete(&mut self) {
        match self.timer_state.phase {
            TimerPhase::Work => {
                // Work complete - go to break
                if self.timer_state.completed_sessions >= self.config.work_sessions_per_cycle {
                    self.start_long_break();
                    self.send_notification("Work complete! Time for a long break.");
                } else {
                    self.start_short_break();
                    self.send_notification("Work complete! Time for a short break.");
                }
                sound::play_break_sound();
            }
            TimerPhase::ShortBreak | TimerPhase::LongBreak => {
                // Break complete - return to Idle, wait for user to click to start next work
                if self.timer_state.completed_sessions >= self.config.work_sessions_per_cycle {
                    self.timer_state.completed_sessions = 0;
                }
                self.timer_state.phase = TimerPhase::Idle;
                self.timer_state.is_paused = false;
                self.timer_state.time_remaining_secs = 0;
                self.send_notification("Break over! Click to start next work session.");
                sound::play_work_sound();
            }
            _ => {}
        }
    }

    /// Sends a notification
    fn send_notification(&self, message: &str) {
        // Silent fail if notification system is unavailable
        let _ = notify_rust::Notification::new()
            .summary("Pomodoro Timer")
            .body(message)
            .icon("appointment-soon")
            .show();
    }
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    ButtonClicked,
    UpdateConfig(Config),
    Tick,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "com.github.axelbdt.cosmodoro";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
        let app = AppModel {
            core,
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            ..Default::default()
        };

        (app, Task::none())
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// The applet's button in the panel shows a single emoji representing the current state.
    fn view(&self) -> Element<'_, Self::Message> {
        let emoji = self.timer_state.phase_emoji();
        // TODO: Add tooltip with time and session count
        let tooltip_text = self.timer_state.tooltip_text();

        button::text(emoji)
            .on_press(Message::ButtonClicked)
            .tooltip(tooltip_text)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-lived async tasks running in the background which
    /// emit messages to the application through a channel. They may be conditionally
    /// activated by selectively appending to the subscription batch, and will
    /// continue to execute for the duration that they remain in the batch.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct TimerTick;

        let mut subscriptions = vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Add timer tick subscription when timer is running
        if !self.timer_state.is_paused
            && self.timer_state.phase != TimerPhase::Idle
            && self.timer_state.time_remaining_secs > 0
        {
            subscriptions.push(Subscription::run_with_id(
                std::any::TypeId::of::<TimerTick>(),
                cosmic::iced::stream::channel(1, move |mut channel| async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        if channel.send(Message::Tick).await.is_err() {
                            break;
                        }
                    }
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime. The application will not exit until all
    /// tasks are finished.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::Tick => {
                // Decrement timer if running
                if !self.timer_state.is_paused
                    && self.timer_state.phase != TimerPhase::Idle
                    && self.timer_state.time_remaining_secs > 0
                {
                    self.timer_state.time_remaining_secs -= 1;

                    // Check if timer completed
                    if self.timer_state.time_remaining_secs == 0 {
                        self.handle_timer_complete();
                    }
                }
            }
            Message::ButtonClicked => {
                match self.timer_state.phase {
                    TimerPhase::Idle => {
                        // Start work session (first or next)
                        self.start_work();
                    }
                    TimerPhase::Work => {
                        if self.timer_state.is_paused {
                            // Resume from pause
                            self.timer_state.is_paused = false;
                            self.send_notification("Timer resumed");
                        } else {
                            // Pause
                            self.timer_state.is_paused = true;
                            self.send_notification("Timer paused");
                        }
                    }
                    TimerPhase::ShortBreak | TimerPhase::LongBreak => {
                        // Skip break and start new work session immediately
                        if self.timer_state.completed_sessions
                            >= self.config.work_sessions_per_cycle
                        {
                            self.timer_state.completed_sessions = 0;
                        }
                        self.start_work();
                        self.send_notification("Break skipped. Starting work session.");
                    }
                }
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
