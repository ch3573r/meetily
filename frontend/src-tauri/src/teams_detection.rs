use serde::{Deserialize, Serialize};
use sysinfo::System;

const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.65;
const PROCESS_SIGNAL_CONFIDENCE: f32 = 0.35;
const BROWSER_PROCESS_SIGNAL_CONFIDENCE: f32 = 0.15;
const TITLE_SIGNAL_CONFIDENCE: f32 = 0.45;
const BROWSER_TITLE_SIGNAL_CONFIDENCE: f32 = 0.35;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsDetectionConfig {
    pub enabled: bool,
    pub confidence_threshold: f32,
    pub require_meeting_title_signal: bool,
    pub max_window_title_samples: usize,
}

impl Default for TeamsDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(target_os = "windows"),
            confidence_threshold: DEFAULT_CONFIDENCE_THRESHOLD,
            require_meeting_title_signal: true,
            max_window_title_samples: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsDetectionStatus {
    pub supported: bool,
    pub enabled: bool,
    pub platform: String,
    pub detected: bool,
    pub confidence: f32,
    pub threshold: f32,
    pub require_meeting_title_signal: bool,
    pub reason: String,
    pub signals: Vec<TeamsDetectionSignal>,
    pub candidates: Vec<TeamsDetectionCandidate>,
    pub next_recommended_action: TeamsDetectionAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsDetectionSignal {
    pub detector: String,
    pub matched: bool,
    pub confidence: f32,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamsDetectionCandidate {
    pub source: String,
    pub process_id: Option<u32>,
    pub process_name: Option<String>,
    pub window_title: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TeamsDetectionAction {
    Idle,
    PromptToRecord,
    Unsupported,
    Disabled,
}

#[derive(Debug, Clone)]
struct ProcessSnapshot {
    pid: u32,
    name: String,
}

#[derive(Debug, Clone)]
struct WindowSnapshot {
    pid: Option<u32>,
    title: String,
}

#[tauri::command]
pub fn get_teams_detection_config() -> TeamsDetectionConfig {
    TeamsDetectionConfig::default()
}

#[tauri::command]
pub fn get_teams_detection_status(config: Option<TeamsDetectionConfig>) -> TeamsDetectionStatus {
    detect_teams_meeting(config.unwrap_or_default())
}

fn detect_teams_meeting(config: TeamsDetectionConfig) -> TeamsDetectionStatus {
    let threshold = normalize_threshold(config.confidence_threshold);

    if !cfg!(target_os = "windows") {
        return TeamsDetectionStatus {
            supported: false,
            enabled: false,
            platform: std::env::consts::OS.to_string(),
            detected: false,
            confidence: 0.0,
            threshold,
            require_meeting_title_signal: config.require_meeting_title_signal,
            reason: "Teams meeting detection is currently implemented for Windows only".to_string(),
            signals: vec![TeamsDetectionSignal {
                detector: "platform".to_string(),
                matched: false,
                confidence: 0.0,
                detail: format!("Unsupported platform: {}", std::env::consts::OS),
            }],
            candidates: Vec::new(),
            next_recommended_action: TeamsDetectionAction::Unsupported,
        };
    }

    if !config.enabled {
        return TeamsDetectionStatus {
            supported: true,
            enabled: false,
            platform: std::env::consts::OS.to_string(),
            detected: false,
            confidence: 0.0,
            threshold,
            require_meeting_title_signal: config.require_meeting_title_signal,
            reason: "Teams meeting detection is disabled by configuration".to_string(),
            signals: vec![TeamsDetectionSignal {
                detector: "config".to_string(),
                matched: false,
                confidence: 0.0,
                detail: "Detection disabled".to_string(),
            }],
            candidates: Vec::new(),
            next_recommended_action: TeamsDetectionAction::Disabled,
        };
    }

    let processes = list_processes();
    let windows = list_visible_windows(config.max_window_title_samples);
    evaluate_snapshots(config, threshold, &processes, &windows)
}

fn evaluate_snapshots(
    config: TeamsDetectionConfig,
    threshold: f32,
    processes: &[ProcessSnapshot],
    windows: &[WindowSnapshot],
) -> TeamsDetectionStatus {
    let teams_processes: Vec<&ProcessSnapshot> = processes
        .iter()
        .filter(|process| is_teams_process(&process.name))
        .collect();
    let browser_processes: Vec<&ProcessSnapshot> = processes
        .iter()
        .filter(|process| is_browser_process(&process.name))
        .collect();

    let teams_titles: Vec<&WindowSnapshot> = windows
        .iter()
        .filter(|window| looks_like_teams_meeting_title(&window.title))
        .collect();
    let browser_teams_titles: Vec<&WindowSnapshot> = teams_titles
        .iter()
        .copied()
        .filter(|window| {
            window
                .pid
                .and_then(|pid| process_name_for_pid(processes, pid))
                .is_some_and(is_browser_process)
        })
        .collect();

    let mut confidence = 0.0;
    let mut signals = Vec::new();

    let teams_process_matched = !teams_processes.is_empty();
    if teams_process_matched {
        confidence += PROCESS_SIGNAL_CONFIDENCE;
    }
    signals.push(TeamsDetectionSignal {
        detector: "teams-process".to_string(),
        matched: teams_process_matched,
        confidence: if teams_process_matched {
            PROCESS_SIGNAL_CONFIDENCE
        } else {
            0.0
        },
        detail: if teams_process_matched {
            format!("{} Teams process(es) found", teams_processes.len())
        } else {
            "No Teams desktop process found".to_string()
        },
    });

    let browser_process_matched = !browser_processes.is_empty();
    if browser_process_matched {
        confidence += BROWSER_PROCESS_SIGNAL_CONFIDENCE;
    }
    signals.push(TeamsDetectionSignal {
        detector: "browser-process".to_string(),
        matched: browser_process_matched,
        confidence: if browser_process_matched {
            BROWSER_PROCESS_SIGNAL_CONFIDENCE
        } else {
            0.0
        },
        detail: if browser_process_matched {
            format!(
                "{} Edge/Chrome browser process(es) found",
                browser_processes.len()
            )
        } else {
            "No Edge/Chrome browser process found".to_string()
        },
    });

    let title_matched = !teams_titles.is_empty();
    if title_matched {
        confidence += TITLE_SIGNAL_CONFIDENCE;
    }
    signals.push(TeamsDetectionSignal {
        detector: "meeting-window-title".to_string(),
        matched: title_matched,
        confidence: if title_matched {
            TITLE_SIGNAL_CONFIDENCE
        } else {
            0.0
        },
        detail: if title_matched {
            format!(
                "{} visible window title(s) look like Teams meetings",
                teams_titles.len()
            )
        } else {
            "No visible window titles look like an active Teams meeting".to_string()
        },
    });

    let browser_title_matched = !browser_teams_titles.is_empty();
    if browser_title_matched && !teams_process_matched {
        confidence += BROWSER_TITLE_SIGNAL_CONFIDENCE;
    }
    signals.push(TeamsDetectionSignal {
        detector: "browser-meeting-title".to_string(),
        matched: browser_title_matched,
        confidence: if browser_title_matched && !teams_process_matched {
            BROWSER_TITLE_SIGNAL_CONFIDENCE
        } else {
            0.0
        },
        detail: if browser_title_matched {
            "Teams meeting title found in Edge/Chrome".to_string()
        } else {
            "No Teams meeting title found in Edge/Chrome".to_string()
        },
    });

    let title_requirement_met = !config.require_meeting_title_signal || title_matched;
    if !title_requirement_met {
        confidence = confidence.min(threshold - 0.01).max(0.0);
    }
    confidence = confidence.min(1.0);
    let detected = confidence >= threshold && title_requirement_met;

    let mut candidates = Vec::new();
    candidates.extend(
        teams_processes
            .iter()
            .map(|process| TeamsDetectionCandidate {
                source: "process".to_string(),
                process_id: Some(process.pid),
                process_name: Some(process.name.clone()),
                window_title: None,
                confidence: PROCESS_SIGNAL_CONFIDENCE,
            }),
    );
    candidates.extend(teams_titles.iter().map(|window| {
        let process_name = window
            .pid
            .and_then(|pid| process_name_for_pid(processes, pid))
            .map(str::to_string);
        TeamsDetectionCandidate {
            source: "window-title".to_string(),
            process_id: window.pid,
            process_name,
            window_title: Some(window.title.clone()),
            confidence: TITLE_SIGNAL_CONFIDENCE,
        }
    }));

    TeamsDetectionStatus {
        supported: cfg!(target_os = "windows"),
        enabled: config.enabled,
        platform: std::env::consts::OS.to_string(),
        detected,
        confidence,
        threshold,
        require_meeting_title_signal: config.require_meeting_title_signal,
        reason: detection_reason(detected, confidence, threshold, title_requirement_met),
        signals,
        candidates,
        next_recommended_action: if detected {
            TeamsDetectionAction::PromptToRecord
        } else {
            TeamsDetectionAction::Idle
        },
    }
}

fn detection_reason(
    detected: bool,
    confidence: f32,
    threshold: f32,
    title_requirement_met: bool,
) -> String {
    if detected {
        return "Teams meeting confidence met the configured threshold".to_string();
    }

    if !title_requirement_met {
        return "Process hints were not enough because no meeting-like window title was found"
            .to_string();
    }

    format!(
        "Teams meeting confidence {:.2} is below threshold {:.2}",
        confidence, threshold
    )
}

fn normalize_threshold(threshold: f32) -> f32 {
    if threshold.is_finite() {
        threshold.clamp(0.0, 1.0)
    } else {
        DEFAULT_CONFIDENCE_THRESHOLD
    }
}

fn list_processes() -> Vec<ProcessSnapshot> {
    let mut system = System::new_all();
    system.refresh_all();
    system
        .processes()
        .iter()
        .map(|(pid, process)| ProcessSnapshot {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn list_visible_windows(max_samples: usize) -> Vec<WindowSnapshot> {
    windows::list_visible_windows(max_samples)
}

#[cfg(not(target_os = "windows"))]
fn list_visible_windows(_max_samples: usize) -> Vec<WindowSnapshot> {
    Vec::new()
}

fn process_name_for_pid(processes: &[ProcessSnapshot], pid: u32) -> Option<&str> {
    processes
        .iter()
        .find(|process| process.pid == pid)
        .map(|process| process.name.as_str())
}

fn is_teams_process(process_name: &str) -> bool {
    let name = normalize_process_name(process_name);
    matches!(
        name.as_str(),
        "teams.exe" | "ms-teams.exe" | "msteams.exe" | "update.exe"
    ) || name.contains("microsoft teams")
}

fn is_browser_process(process_name: &str) -> bool {
    let name = normalize_process_name(process_name);
    matches!(
        name.as_str(),
        "msedge.exe" | "chrome.exe" | "msedgewebview2.exe"
    )
}

fn normalize_process_name(process_name: &str) -> String {
    process_name.trim().to_lowercase()
}

fn looks_like_teams_meeting_title(title: &str) -> bool {
    let title = title.trim().to_lowercase();

    if title.is_empty() {
        return false;
    }

    let has_teams_context = title.contains("teams")
        || title.contains("microsoft teams")
        || title.contains("teams.microsoft.com");
    let has_meeting_context = title.contains("meeting")
        || title.contains("call")
        || title.contains("joined")
        || title.contains("lobby")
        || title.contains("screen sharing")
        || title.contains("presenting")
        || title.contains("participants")
        || title.contains("mute")
        || title.contains("unmute")
        || title.contains("leave");

    has_teams_context && has_meeting_context
}

#[cfg(target_os = "windows")]
mod windows {
    use super::WindowSnapshot;
    use std::ffi::c_void;

    type Bool = i32;
    type Dword = u32;
    type Hwnd = *mut c_void;
    type Lparam = isize;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(
            enum_func: Option<unsafe extern "system" fn(Hwnd, Lparam) -> Bool>,
            lparam: Lparam,
        ) -> Bool;
        fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
        fn GetWindowTextW(hwnd: Hwnd, string: *mut u16, max_count: i32) -> i32;
        fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut Dword) -> Dword;
        fn IsWindowVisible(hwnd: Hwnd) -> Bool;
    }

    struct WindowCollector {
        max_samples: usize,
        windows: Vec<WindowSnapshot>,
    }

    pub fn list_visible_windows(max_samples: usize) -> Vec<WindowSnapshot> {
        let mut collector = WindowCollector {
            max_samples,
            windows: Vec::new(),
        };

        unsafe {
            EnumWindows(
                Some(enum_window),
                &mut collector as *mut WindowCollector as Lparam,
            );
        }

        collector.windows
    }

    unsafe extern "system" fn enum_window(hwnd: Hwnd, lparam: Lparam) -> Bool {
        let collector = &mut *(lparam as *mut WindowCollector);

        if collector.windows.len() >= collector.max_samples {
            return 0;
        }

        if IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let title_length = GetWindowTextLengthW(hwnd);
        if title_length <= 0 {
            return 1;
        }

        let mut buffer = vec![0u16; title_length as usize + 1];
        let copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        if copied <= 0 {
            return 1;
        }

        let title = String::from_utf16_lossy(&buffer[..copied as usize])
            .trim()
            .to_string();
        if title.is_empty() {
            return 1;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        collector.windows.push(WindowSnapshot {
            pid: (process_id != 0).then_some(process_id),
            title,
        });

        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_detection_requires_teams_and_meeting_context() {
        assert!(looks_like_teams_meeting_title(
            "Weekly sync | Microsoft Teams Meeting"
        ));
        assert!(looks_like_teams_meeting_title(
            "Budget review - Teams - Call in progress"
        ));
        assert!(!looks_like_teams_meeting_title("Microsoft Teams"));
        assert!(!looks_like_teams_meeting_title("Calendar - meeting notes"));
    }

    #[test]
    fn confidence_detects_desktop_meeting_window() {
        let config = TeamsDetectionConfig::default();
        let processes = vec![ProcessSnapshot {
            pid: 42,
            name: "ms-teams.exe".to_string(),
        }];
        let windows = vec![WindowSnapshot {
            pid: Some(42),
            title: "Weekly sync | Microsoft Teams Meeting".to_string(),
        }];

        let status = evaluate_snapshots(config, DEFAULT_CONFIDENCE_THRESHOLD, &processes, &windows);

        assert!(status.detected);
        assert!(status.confidence >= DEFAULT_CONFIDENCE_THRESHOLD);
        assert_eq!(
            status.next_recommended_action,
            TeamsDetectionAction::PromptToRecord
        );
    }

    #[test]
    fn process_only_is_not_detected_when_title_signal_is_required() {
        let config = TeamsDetectionConfig::default();
        let processes = vec![ProcessSnapshot {
            pid: 42,
            name: "ms-teams.exe".to_string(),
        }];

        let status = evaluate_snapshots(config, DEFAULT_CONFIDENCE_THRESHOLD, &processes, &[]);

        assert!(!status.detected);
        assert!(status.confidence < DEFAULT_CONFIDENCE_THRESHOLD);
    }

    #[test]
    fn browser_meeting_title_can_reach_threshold() {
        let config = TeamsDetectionConfig::default();
        let processes = vec![ProcessSnapshot {
            pid: 99,
            name: "msedge.exe".to_string(),
        }];
        let windows = vec![WindowSnapshot {
            pid: Some(99),
            title: "teams.microsoft.com - Customer call - Microsoft Teams".to_string(),
        }];

        let status = evaluate_snapshots(config, DEFAULT_CONFIDENCE_THRESHOLD, &processes, &windows);

        assert!(status.detected);
    }
}
