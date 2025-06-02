/// LED Memory Display Component for MIPS Emulator
#[derive(Clone)]
pub struct MemoryRow {
    pub address: u32,
    pub data: u32,
}

impl MemoryRow {
    pub fn new(address: u32, data: u32) -> Self {
        Self { address, data }
    }

    /// Get the bit at the specified position (0-31, where 0 is LSB)
    pub fn get_bit(&self, bit_index: usize) -> bool {
        if bit_index < 32 {
            (self.data >> bit_index) & 1 == 1
        } else {
            false
        }
    }

    /// Set the bit at the specified position
    pub fn set_bit(&mut self, bit_index: usize, value: bool) {
        if bit_index < 32 {
            if value {
                self.data |= 1 << bit_index;
            } else {
                self.data &= !(1 << bit_index);
            }
        }
    }
}

/// MIPS Emulator App with LED Memory Display
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    #[serde(skip)] // Don't serialize the memory rows for now
    memory_rows: Vec<MemoryRow>,
    
    // UI state
    num_rows: usize,
    led_size: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let mut app = Self {
            memory_rows: Vec::new(),
            num_rows: 4, // Default to 8 rows
            led_size: 12.0,
        };
        
        // Initialize with some default memory rows
        app.initialize_memory_rows();
        app
    }
}

impl TemplateApp {
    /// Initialize memory rows starting from address 0x00000000
    fn initialize_memory_rows(&mut self) {
        self.memory_rows.clear();
        for i in 0..self.num_rows {
            let address = (i as u32) * 4; // Word-aligned addresses
            let data = 0; // Initialize with zeros
            self.memory_rows.push(MemoryRow::new(address, data));
        }
    }

    /// Add a new memory row
    pub fn add_memory_row(&mut self) {
        let address = (self.memory_rows.len() as u32) * 4;
        self.memory_rows.push(MemoryRow::new(address, 0));
        self.num_rows = self.memory_rows.len();
    }

    /// Remove the last memory row
    pub fn remove_memory_row(&mut self) {
        if !self.memory_rows.is_empty() {
            self.memory_rows.pop();
            self.num_rows = self.memory_rows.len();
        }
    }

    /// Set data for a specific memory address
    pub fn set_memory_data(&mut self, address: u32, data: u32) {
        if let Some(row) = self.memory_rows.iter_mut().find(|r| r.address == address) {
            row.data = data;
        }
    }

    /// Get data from a specific memory address
    pub fn get_memory_data(&self, address: u32) -> Option<u32> {
        self.memory_rows.iter().find(|r| r.address == address).map(|r| r.data)
    }

    /// Load data from an array into memory starting at address 0x00000000
    pub fn load_memory_from_array(&mut self, data: &[u32]) {
        // Ensure we have enough rows
        while self.memory_rows.len() < data.len() {
            self.add_memory_row();
        }
        
        // Load the data
        for (i, &value) in data.iter().enumerate() {
            if i < self.memory_rows.len() {
                self.memory_rows[i].data = value;
            }
        }
    }

    /// Clear all memory (set all bits to 0)
    pub fn clear_memory(&mut self) {
        for row in &mut self.memory_rows {
            row.data = 0;
        }
    }

    /// Set a test pattern (alternating bits)
    pub fn set_test_pattern(&mut self) {
        for (i, row) in self.memory_rows.iter_mut().enumerate() {
            // Alternate between 0xAAAAAAAA and 0x55555555
            row.data = if i % 2 == 0 { 0xAAAAAAAA } else { 0x55555555 };
        }
    }

    /// Draw a memory row with 32 LEDs
    fn draw_memory_row(&mut self, ui: &mut egui::Ui, row_index: usize) {
        if row_index >= self.memory_rows.len() {
            return;
        }

        let row = &mut self.memory_rows[row_index];
        
        ui.horizontal(|ui| {
            // Display memory address
            ui.label(format!("0x{:08X}:", row.address));
            ui.add_space(10.0);
            
            // Draw 32 LEDs (bits 31 to 0, left to right)
            for bit_index in (0..32).rev() {
                let is_on = row.get_bit(bit_index);
                
                // Make LED clickable to toggle bit
                let size = egui::Vec2::splat(self.led_size);
                let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                
                if response.clicked() {
                    row.set_bit(bit_index, !is_on);
                }
                
                let color = if is_on {
                    egui::Color32::from_rgb(255, 0, 0) // Red when on
                } else {
                    egui::Color32::from_rgb(64, 64, 64) // Dark gray when off
                };
                
                ui.painter().circle_filled(rect.center(), self.led_size / 2.0, color);
                
                // Add a subtle border
                ui.painter().circle_stroke(
                    rect.center(),
                    self.led_size / 2.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(128, 128, 128)),
                );
                
                // Add some spacing between LEDs
                ui.add_space(2.0);
            }
            
            // Display hex value
            ui.add_space(10.0);
            ui.label(format!("0x{:08X}", row.data));
        });
    }


    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("MIPS Emulator - LED Memory Display");

            ui.separator();

            // Controls for managing memory rows
            ui.horizontal(|ui| {
                if ui.button("Add Row").clicked() {
                    self.add_memory_row();
                }
                if ui.button("Remove Row").clicked() {
                    self.remove_memory_row();
                }
                ui.separator();
                if ui.button("Clear All").clicked() {
                    self.clear_memory();
                }
                if ui.button("Test Pattern").clicked() {
                    self.set_test_pattern();
                }
                ui.separator();
                ui.label("LED Size:");
                ui.add(egui::Slider::new(&mut self.led_size, 8.0..=20.0).text("px"));
            });

            ui.separator();

            // Display memory rows with LEDs
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in 0..self.memory_rows.len() {
                    self.draw_memory_row(ui, i);
                    ui.add_space(5.0);
                }
            });

            ui.separator();

            // Instructions
            ui.label("Instructions:");
            ui.label("• Click on any LED to toggle its state (red = 1, gray = 0)");
            ui.label("• Each row represents a 32-bit memory word");
            ui.label("• Memory addresses start at 0x00000000 and increment by 4");
            ui.label("• Use Add/Remove Row buttons to manage memory size");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

// fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
//     ui.horizontal(|ui| {
//         ui.spacing_mut().item_spacing.x = 0.0;
//         ui.label("Powered by ");
//         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
//         ui.label(" and ");
//         ui.hyperlink_to(
//             "eframe",
//             "https://github.com/emilk/egui/tree/master/crates/eframe",
//         );
//         ui.label(".");
//     });
// }
