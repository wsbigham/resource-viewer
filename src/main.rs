#![windows_subsystem = "windows"]
use eframe::egui;
use egui::Color32;
use egui_plot::{Line, Plot, PlotPoints, PlotBounds};
use sysinfo::{MemoryRefreshKind, RefreshKind, CpuRefreshKind, System, Networks, Disks};
use std::collections::VecDeque;
use std::time::{Instant, Duration};

const HISTORY_LENGTH: usize = 60; // 60 seconds of history

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 750.0])
            .with_always_on_top()
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Resource View",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.global_style()).clone();
            style.visuals.window_fill = Color32::from_rgb(20, 20, 24);
            style.visuals.panel_fill = Color32::from_rgb(20, 20, 24);
            style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(30, 30, 36);
            style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(220, 220, 230);
            style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 55);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 85);
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 75);
            cc.egui_ctx.set_global_style(style);
            
            Ok(Box::new(ResourceApp::new()))
        }),
    )
}

struct ResourceApp {
    sys: System,
    networks: Networks,
    disks: Disks,
    
    cpu_history: VecDeque<f64>,
    mem_history: VecDeque<f64>,
    net_rx_history: VecDeque<f64>,
    net_tx_history: VecDeque<f64>,
    disk_usage_history: VecDeque<f64>,
    
    last_update: Instant,
}

impl ResourceApp {
    fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_all();
        
        let mut empty_history: VecDeque<f64> = VecDeque::new();
        for _ in 0..HISTORY_LENGTH {
            empty_history.push_back(0.0);
        }

        Self {
            sys,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            
            cpu_history: empty_history.clone(),
            mem_history: empty_history.clone(),
            net_rx_history: empty_history.clone(),
            net_tx_history: empty_history.clone(),
            disk_usage_history: empty_history,
            
            last_update: Instant::now(),
        }
    }
    
    fn render_plot(ui: &mut egui::Ui, name: &str, history: &VecDeque<f64>, color: Color32, max_val: Option<f64>) {
        let points: Vec<[f64; 2]> = history
            .iter()
            .enumerate()
            .map(|(i, &val)| [i as f64 - HISTORY_LENGTH as f64, val])
            .collect();
            
        let line = Line::new(name, PlotPoints::new(points)).color(color).width(2.0);
        
        Plot::new(name)
            .height(80.0)
            .show_x(false)
            .show_y(false)
            .show_axes(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .include_y(0.0)
            .include_y(max_val.unwrap_or(1.0))
            .show(ui, |plot_ui| plot_ui.line(line));
    }
}

impl eframe::App for ResourceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(1000) {
            self.sys.refresh_cpu_usage();
            self.sys.refresh_memory();
            self.networks.refresh(true);
            self.disks.refresh(true);
            
            let cpu_count = self.sys.cpus().len() as f32;
            let total_cpu: f64 = if cpu_count > 0.0 {
                (self.sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count) as f64
            } else {
                0.0
            };
            self.cpu_history.pop_front();
            self.cpu_history.push_back(total_cpu);
            
            let used_mem = self.sys.used_memory() as f64 / 1_048_576.0;
            self.mem_history.pop_front();
            self.mem_history.push_back(used_mem);
            
            let mut total_rx = 0.0;
            let mut total_tx = 0.0;
            for (_interface_name, data) in &self.networks {
                total_rx += data.received() as f64;
                total_tx += data.transmitted() as f64;
            }
            self.net_rx_history.pop_front();
            self.net_rx_history.push_back(total_rx);
            self.net_tx_history.pop_front();
            self.net_tx_history.push_back(total_tx);
            
            let mut used_disk = 0.0;
            for disk in &self.disks {
                used_disk += (disk.total_space() - disk.available_space()) as f64;
            }
            let used_disk_mb = used_disk / 1_048_576.0;
            self.disk_usage_history.pop_front();
            self.disk_usage_history.push_back(used_disk_mb);
            
            self.last_update = now;
        }

        ui.ctx().request_repaint_after(Duration::from_millis(100));
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);
            
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("System Resources").size(24.0).strong().color(Color32::from_rgb(250, 250, 250)));
            });
            ui.add_space(15.0);
            
            let cpu_val = *self.cpu_history.back().unwrap_or(&0.0);
            ui.label(egui::RichText::new(format!("CPU Usage: {:.1}%", cpu_val)).size(16.0));
            Self::render_plot(ui, "cpu_plot", &self.cpu_history, Color32::from_rgb(80, 200, 120), Some(100.0));
            ui.add_space(10.0);
            
            let mem_val = *self.mem_history.back().unwrap_or(&0.0);
            let total_mem = self.sys.total_memory() as f64 / 1_048_576.0;
            ui.label(egui::RichText::new(format!("Memory Usage: {:.0} MB / {:.0} MB", mem_val, total_mem)).size(16.0));
            Self::render_plot(ui, "mem_plot", &self.mem_history, Color32::from_rgb(100, 150, 250), Some(total_mem));
            ui.add_space(10.0);
            
            let rx_val = *self.net_rx_history.back().unwrap_or(&0.0);
            let tx_val = *self.net_tx_history.back().unwrap_or(&0.0);
            ui.label(egui::RichText::new(format!("Network (B/s) \nRX: {:.0} | TX: {:.0}", rx_val, tx_val)).size(16.0));
            
            let rx_points: Vec<[f64; 2]> = self.net_rx_history.iter().enumerate().map(|(i, &v)| [i as f64 - HISTORY_LENGTH as f64, v]).collect();
            let tx_points: Vec<[f64; 2]> = self.net_tx_history.iter().enumerate().map(|(i, &v)| [i as f64 - HISTORY_LENGTH as f64, v]).collect();
            let rx_line = Line::new("RX", PlotPoints::new(rx_points)).color(Color32::from_rgb(250, 100, 100)).width(2.0);
            let tx_line = Line::new("TX", PlotPoints::new(tx_points)).color(Color32::from_rgb(100, 200, 250)).width(2.0);
            
            Plot::new("net_plot")
                .height(80.0)
                .show_x(false)
                .show_y(false)
                .show_axes(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .include_y(0.0)
                .show(ui, |plot_ui| {
                    plot_ui.line(rx_line);
                    plot_ui.line(tx_line);
                });
            ui.add_space(10.0);
            
            let disk_val = *self.disk_usage_history.back().unwrap_or(&0.0);
            let mut total_disk = 0.0;
            for disk in &self.disks { total_disk += disk.total_space() as f64; }
            let total_disk_mb: f64 = total_disk / 1_048_576.0;
            ui.label(egui::RichText::new(format!("Total Disk Used: {:.0} MB / {:.0} MB", disk_val, total_disk_mb)).size(16.0));
            Self::render_plot(ui, "disk_plot", &self.disk_usage_history, Color32::from_rgb(200, 200, 80), Some(total_disk_mb));
        });
    }
}
