//! Hotkey system
//!
//! Centralized hotkey management for the NLA editor.
//!
//! # Architecture
//! 
//! - **HotkeyAction**: Enum of all possible actions that can be triggered by hotkeys
//! - **HotkeyContext**: Determines which hotkeys are active based on app state
//! - **handle_hotkey()**: Main dispatch function that maps key events to actions
//!
//! # Adding New Hotkeys
//!
//! 1. Add a variant to `HotkeyAction`
//! 2. Add the key binding in `key_to_action()`
//! 3. Handle the action in the App component's hotkey handler

use dioxus::prelude::Key;

/// All possible actions that can be triggered by hotkeys.
/// 
/// Each variant represents a semantic action, not a key binding.
/// This decouples "what key was pressed" from "what should happen".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    // ═══════════════════════════════════════════════════════════════
    // Timeline Zoom
    // ═══════════════════════════════════════════════════════════════
    /// Zoom in on the timeline (increase pixels per second)
    TimelineZoomIn,
    /// Zoom out on the timeline (decrease pixels per second)
    TimelineZoomOut,
    /// Save the current project.
    SaveProject,
    /// Toggle playback.
    PlayPause,

    // ═══════════════════════════════════════════════════════════════
    // Playback (future)
    // ═══════════════════════════════════════════════════════════════
    // PlayPause,
    // SeekStart,
    // SeekEnd,
    // StepForward,
    // StepBackward,

    // ═══════════════════════════════════════════════════════════════
    // Selection (future)
    // ═══════════════════════════════════════════════════════════════
    // DeleteSelection,
    // SelectAll,
    // DeselectAll,
}

/// Context information that affects which hotkeys are active.
/// 
/// Some hotkeys only make sense in certain contexts:
/// - Timeline zoom requires the timeline to be visible
/// - Delete requires a selection
/// - Etc.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct HotkeyContext {
    /// Whether the timeline panel is visible (not collapsed)
    #[allow(dead_code)]
    pub timeline_visible: bool,
    /// Whether any clips are selected
    #[allow(dead_code)]
    pub has_selection: bool,
    /// Whether an input field has focus (should suppress most hotkeys)
    pub input_focused: bool,
}

/// Result of processing a key event.
#[derive(Debug, Clone)]
pub enum HotkeyResult {
    /// A hotkey action was matched and should be executed
    Action(HotkeyAction),
    /// No matching hotkey for this key/context combination
    NoMatch,
    /// Hotkey would match but is suppressed (e.g., input field focused)
    Suppressed,
}

/// Maps a key event to an action, considering the current context.
///
/// # Arguments
/// * `key` - The key that was pressed
/// * `modifiers` - Modifier keys held (shift, ctrl, alt, meta)
/// * `context` - Current application context
///
/// # Returns
/// * `HotkeyResult::Action(action)` if a hotkey matched
/// * `HotkeyResult::NoMatch` if no binding exists
/// * `HotkeyResult::Suppressed` if input is focused
pub fn handle_hotkey(
    key: &Key,
    _shift: bool,
    ctrl: bool,
    _alt: bool,
    meta: bool,
    context: &HotkeyContext,
) -> HotkeyResult {
    // Suppress hotkeys when typing in an input field
    if context.input_focused {
        return HotkeyResult::Suppressed;
    }

    // ═══════════════════════════════════════════════════════════════
    // Global Hotkeys (work regardless of context)
    // ═══════════════════════════════════════════════════════════════
    
    // Timeline zoom: Numpad +/- (produces "+" and "-" characters)
    // Also handles regular +/- for convenience
    match key {
        Key::Character(c) if (ctrl || meta) && (c == "s" || c == "S") => {
            return HotkeyResult::Action(HotkeyAction::SaveProject);
        }
        Key::Character(c) if c == "+" => return HotkeyResult::Action(HotkeyAction::TimelineZoomIn),
        Key::Character(c) if c == "-" => return HotkeyResult::Action(HotkeyAction::TimelineZoomOut),
        Key::Character(c) if c == " " => return HotkeyResult::Action(HotkeyAction::PlayPause),
        _ => {}
    }

    // ═══════════════════════════════════════════════════════════════
    // Context-Specific Hotkeys
    // ═══════════════════════════════════════════════════════════════
    
    // (Future: Add context-aware hotkeys here)
    // Example:
    // if context.has_selection {
    //     match key {
    //         Key::Delete => return HotkeyResult::Action(HotkeyAction::DeleteSelection),
    //         _ => {}
    //     }
    // }

    HotkeyResult::NoMatch
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plus_zooms_in() {
        let ctx = HotkeyContext::default();
        let result = handle_hotkey(&Key::Character("+".to_string()), false, false, false, false, &ctx);
        assert!(matches!(result, HotkeyResult::Action(HotkeyAction::TimelineZoomIn)));
    }

    #[test]
    fn test_minus_zooms_out() {
        let ctx = HotkeyContext::default();
        let result = handle_hotkey(&Key::Character("-".to_string()), false, false, false, false, &ctx);
        assert!(matches!(result, HotkeyResult::Action(HotkeyAction::TimelineZoomOut)));
    }

    #[test]
    fn test_ctrl_s_saves_project() {
        let ctx = HotkeyContext::default();
        let result = handle_hotkey(&Key::Character("s".to_string()), false, true, false, false, &ctx);
        assert!(matches!(result, HotkeyResult::Action(HotkeyAction::SaveProject)));
    }

    #[test]
    fn test_space_toggles_playback() {
        let ctx = HotkeyContext::default();
        let result = handle_hotkey(&Key::Character(" ".to_string()), false, false, false, false, &ctx);
        assert!(matches!(result, HotkeyResult::Action(HotkeyAction::PlayPause)));
    }

    #[test]
    fn test_suppressed_when_input_focused() {
        let ctx = HotkeyContext {
            input_focused: true,
            ..Default::default()
        };
        let result = handle_hotkey(&Key::Character("+".to_string()), false, false, false, false, &ctx);
        assert!(matches!(result, HotkeyResult::Suppressed));
    }
}

