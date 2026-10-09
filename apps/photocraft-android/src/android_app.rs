//! Android application runtime and eframe runner setup.

use std::path::PathBuf;
use std::sync::Arc;

use photocraft_doc::Document;
use photocraft_engine::Session;
use photocraft_ui_egui::{PhotocraftApp, Services};

use crate::mobile;

/// Starts the PhotoCraft Android runner.
pub fn start(android_app: android_activity::AndroidApp) {
    let internal_path = android_app.internal_data_path().map(PathBuf::from);

    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(android_app);

    // Default to wgpu with Vulkan backend on Android.
    photocraft_ui_egui::gpu_canvas::use_adapter_limits(&mut options.wgpu_options.wgpu_setup);

    let result = eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(move |cc| {
            photocraft_ui_egui::PhotocraftApp::setup_context(&cc.egui_ctx, photocraft_ui_egui::theme::ThemeKind::Pro);
            mobile::configure_mobile_style(&mut cc.egui_ctx.style_mut());

            let services = create_mobile_services(internal_path.clone());
            let mut app = PhotocraftApp::new(Session::new(), services);

            // Apply phone layout defaults (canvas-first, collapsed dock).
            mobile::apply_mobile_defaults(&mut app);

            if let Some(rs) = cc.wgpu_render_state.clone() {
                log::info!("photocraft-android: wgpu backend {:?}", rs.adapter.get_info().backend);
                app.set_wgpu(rs);
            }

            Ok(Box::new(MobileShell { app }))
        }),
    );

    if let Err(e) = result {
        log::error!("PhotoCraft failed to run on Android: {e}");
    }
}

/// The Android application wrapper.
struct MobileShell {
    app: PhotocraftApp,
}

impl eframe::App for MobileShell {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.logic(ctx, frame);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
    }
}

/// Creates platform services adapted for the Android app sandbox.
fn create_mobile_services(storage_dir: Option<PathBuf>) -> Services {
    let mut services = Services::default();

    if let Some(dir) = storage_dir {
        let prefs_dir = dir.join("prefs");
        let _ = std::fs::create_dir_all(&prefs_dir);
        let prefs_file = prefs_dir.join("photocraft_prefs.json");

        let read_path = prefs_file.clone();
        services.load_prefs = Some(Arc::new(move || std::fs::read_to_string(&read_path).ok()));

        let write_path = prefs_file;
        services.save_prefs = Some(Arc::new(move |content: &str| {
            let _ = std::fs::write(&write_path, content);
        }));
    }

    // Import service for decoding image formats.
    services.import = Some(Box::new(|name: &str, bytes: &[u8]| {
        photocraft_io::import(name, bytes)
            .map(|r| (r.document, r.warnings))
            .map_err(|e| e.to_string())
    }));

    // Export service for saving images.
    services.export = Some(Box::new(|doc: &Document, path: &str, settings: &photocraft_ui_egui::ExportSettings| {
        let mut opts = photocraft_io::ExportOptions::default();
        if let Some(q) = settings.jpeg_quality {
            opts.encode.jpeg_quality = q;
        }
        opts.encode.webp_lossless = settings.webp_lossless;
        if let Some(q) = settings.webp_quality {
            opts.encode.webp_quality = q;
        }
        opts.tiff_layers = settings.tiff_layers;
        opts.xmp = if settings.xmp_all {
            photocraft_io::XmpEmbed::All
        } else {
            photocraft_io::XmpEmbed::None
        };
        photocraft_io::export(doc, path, &opts)
            .map(|r| (r.bytes, r.warnings))
            .map_err(|e| e.to_string())
    }));

    services
}

