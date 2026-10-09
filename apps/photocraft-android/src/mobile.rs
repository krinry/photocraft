//! Mobile UI adaptations and touch-first configuration for PhotoCraft on phone screens.
//!
//! Phones differ significantly from desktop monitors:
//! - Physical screen space is limited (360–430 pt typical width in portrait);
//! - Dense desktop panels (Layers, Channels, Navigator) reduce canvas area to unusable sizes if kept open;
//! - Finger touch targets require larger interactive hitboxes (~44 pt minimum);
//! - Modifiers (Shift, Alt, Ctrl) are not physically available without an on-screen helper.
//!
//! This module optimizes layout defaults and interaction ergonomics for phone screens.

use photocraft_ui_egui::PhotocraftApp;

/// Configures egui style spacing and touch target sizing for phones.
pub fn configure_mobile_style(style: &mut egui::Style) {
    // Increase interactive target bounds to comfortably match fingers (standard 44pt touch targets).
    style.spacing.interact_size = egui::vec2(44.0, 40.0);
    style.spacing.button_padding = egui::vec2(12.0, 10.0);
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.icon_width = 24.0;
}

/// Applies phone-first UI state defaults to maximize canvas working space.
pub fn apply_mobile_defaults(app: &mut PhotocraftApp) {
    // 1. Collapse the right-hand dock (Layers/Channels) by default.
    // Users can toggle it from the top bar or via gesture, leaving full screen width for the canvas.
    app.ui.panels.dock = false;

    // 2. Hide desktop status bar (zoom % and document info strip) to save vertical screen space.
    app.ui.panels.status_bar = false;

    // 3. Keep essential editing controls visible.
    app.ui.panels.toolbar = true;
    app.ui.panels.options_bar = true;

    // 4. Enable on-screen modifier keys (Shift, Ctrl, Alt) so touch/pen users can perform
    // additive selections, straight lines, and duplicate transforms without a hardware keyboard.
    app.ui.shell.modifier_keys = true;

    // 5. System status bar is handled by Android; disable desktop custom title bar decor.
    app.custom_titlebar = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use photocraft_engine::Session;
    use photocraft_ui_egui::Services;

    #[test]
    fn mobile_defaults_maximize_canvas_space() {
        let mut app = PhotocraftApp::new(Session::new(), Services::default());

        // In standard desktop mode, dock and status bar are active.
        assert!(app.ui.panels.dock);
        assert!(app.ui.panels.status_bar);

        // Apply mobile phone adaptations.
        apply_mobile_defaults(&mut app);

        // Verify phone optimizations:
        assert!(!app.ui.panels.dock, "Dock must be collapsed on phones to give space to canvas");
        assert!(!app.ui.panels.status_bar, "Status bar must be hidden to maximize vertical space");
        assert!(app.ui.panels.toolbar, "Toolbar should remain available");
        assert!(app.ui.shell.modifier_keys, "Modifier keys panel must be enabled for touch input");
        assert!(!app.custom_titlebar, "Custom window title bar is disabled on Android");
    }

    #[test]
    fn mobile_style_sets_finger_friendly_dimensions() {
        let mut style = egui::Style::default();
        configure_mobile_style(&mut style);

        assert!(style.spacing.interact_size.x >= 44.0, "Touch width target must be >= 44pt");
        assert!(style.spacing.interact_size.y >= 36.0, "Touch height target must be >= 36pt");
        assert!(style.spacing.button_padding.x >= 10.0, "Button horizontal padding must be comfortable");
    }
}

