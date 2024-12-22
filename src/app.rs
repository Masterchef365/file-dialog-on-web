use std::{path::PathBuf, sync::Arc};

use egui_file_dialog::{Disk, Disks, FileDialog, FileSystem, Metadata};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct TemplateApp {
    path: String,

    #[serde(skip)]
    dialog: FileDialog,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut path = String::new();
        if let Some(storage) = cc.storage {
            path = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            path,
            dialog: FileDialog::from_filesystem(Arc::new(MyFileSystem)),
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.path);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Pick").clicked() {
                self.dialog.pick_file()
            }
            ui.label(&self.path);

            if let Some(path) = self.dialog.update(ctx).picked() {
                self.path = path.to_string_lossy().to_string();
            }
        });
    }
}

pub struct MyFileSystem;

impl FileSystem for MyFileSystem {
    fn is_dir(&self, path: &std::path::Path) -> bool {
        !self.is_file(path)
    }

    fn is_file(&self, path: &std::path::Path) -> bool {
        true
    }

    fn metadata(&self, path: &std::path::Path) -> std::io::Result<Metadata> {
        Ok(Metadata::default())
    }

    fn read_dir(&self, path: &std::path::Path) -> std::io::Result<Vec<PathBuf>> {
        Ok((0..30).map(|i| PathBuf::from(format!("{i}.txt"))).collect())
    }

    fn user_dirs(&self, canonicalize_paths: bool) -> Option<egui_file_dialog::UserDirectories> {
        Some(egui_file_dialog::UserDirectories {
            home_dir: Some("Certified".into()),
            audio_dir: Some("Hood".into()),
            desktop_dir: Some("Classic".into()),
            document_dir: None,
            download_dir: None,
            picture_dir: None,
            video_dir: None,
        })
    }

    fn get_disks(&self, canonicalize_paths: bool) -> egui_file_dialog::Disks {
        Disks {
            disks: vec![
                Disk {
                    is_removable: false,
                    mount_point: "/dev/nvmeeeee".into(),
                    display_name: "/dev/nvmeeeee".into(),
                },
                Disk {
                    is_removable: false,
                    display_name: "C:\\wunkus".into(),
                    mount_point: "C:\\wunkus".into(),
                },
            ],
        }
    }

    fn create_dir(&self, path: &std::path::Path) -> std::io::Result<()> {
        Ok(())
    }

    fn is_path_hidden(&self, path: &std::path::Path) -> bool {
        false
    }

    fn load_text_file_preview(
        &self,
        path: &std::path::Path,
        max_chars: usize,
    ) -> std::io::Result<String> {
        Ok("Hello world!".to_string())
    }
}
