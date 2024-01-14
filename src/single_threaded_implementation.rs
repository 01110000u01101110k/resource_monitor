use std::time::{Duration, Instant};

use eframe::egui::{self, Event, Vec2};
use egui_plot::{Legend, Line, PlotPoints};
use crate::cpu_temperature::{get_cpu_current_celsius_temperature_using_wmi};
use crate::gpu_temperature::{get_gpu_current_celsius_temperature, get_gpu_current_celsius_temperature_nvml};

use windows::{
    Win32::System::Com::*,
    Win32::System::Wmi::*,
};

use nvml_wrapper::Nvml;

/*
struct CpuInfo {
    name: String,
    temperature: u16,
}

struct GpuInfo {
    name: String,
    temperature: u16,
}
*/

struct PlotExample {
    lock_x: bool,
    lock_y: bool,
    ctrl_to_zoom: bool,
    shift_to_horizontal: bool,
    zoom_speed: f32,
    scroll_speed: f32,
    time_between_update: Duration,
    inner_timer: Option<Instant>,
    gpu_temperature: Vec<f32>,
    cpu_temperature: Vec<f32>,
    wmi_server: IWbemServices,
    nvml: Nvml
}

impl Default for PlotExample {
    fn default() -> Self {
        unsafe {
            /*
                ініціалізую інтерфейс IWbemServices, він використовується 
                клієнтами та постачальниками для доступу до служб WMI
            */
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).unwrap();
            let wmi_server = locator.ConnectServer(&windows::core::BSTR::from("root\\cimv2"), None, None, None, 0, None, None).unwrap();

            // ініціалізую nvml
            let nvml = Nvml::init().unwrap();

            Self {
                lock_x: false,
                lock_y: false,
                ctrl_to_zoom: false,
                shift_to_horizontal: false,
                zoom_speed: 1.0,
                scroll_speed: 1.0,
                time_between_update: Duration::from_millis(500),
                inner_timer: None,
                gpu_temperature: Vec::new(),
                cpu_temperature: Vec::new(),
                wmi_server,
                nvml
            }
        }
    }
}

impl eframe::App for PlotExample {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            egui::SidePanel::left("options").show(&ctx, |ui| {
                ui.checkbox(&mut self.lock_x, "Lock x axis").on_hover_text("Check to keep the X axis fixed, i.e., pan and zoom will only affect the Y axis");
                ui.checkbox(&mut self.lock_y, "Lock y axis").on_hover_text("Check to keep the Y axis fixed, i.e., pan and zoom will only affect the X axis");
                ui.checkbox(&mut self.ctrl_to_zoom, "Ctrl to zoom").on_hover_text("If unchecked, the behavior of the Ctrl key is inverted compared to the default controls\ni.e., scrolling the mouse without pressing any keys zooms the plot");
                ui.checkbox(&mut self.shift_to_horizontal, "Shift for horizontal scroll").on_hover_text("If unchecked, the behavior of the shift key is inverted compared to the default controls\ni.e., hold to scroll vertically, release to scroll horizontally");
                
            });
            egui::CentralPanel::default().show(&ctx, |ui| {
                let (scroll, pointer_down, modifiers) = ui.input(|i| {
                    let scroll = i.events.iter().find_map(|e| match e {
                        Event::MouseWheel {
                            unit: _,
                            delta,
                            modifiers: _,
                        } => Some(*delta),
                        _ => None,
                    });
                    (scroll, i.pointer.primary_down(), i.modifiers)
                });

                ui.label("графік температур процесора та відеокарти");

                egui_plot::Plot::new("resource_monitor")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        if let Some(mut scroll) = scroll {
                            if modifiers.ctrl == self.ctrl_to_zoom {
                                scroll = Vec2::splat(scroll.x + scroll.y);
                                let mut zoom_factor = Vec2::from([
                                    (scroll.x * self.zoom_speed / 10.0).exp(),
                                    (scroll.y * self.zoom_speed / 10.0).exp(),
                                ]);
                                if self.lock_x {
                                    zoom_factor.x = 1.0;
                                }
                                if self.lock_y {
                                    zoom_factor.y = 1.0;
                                }
                                plot_ui.zoom_bounds_around_hovered(zoom_factor);
                            } else {
                                if modifiers.shift == self.shift_to_horizontal {
                                    scroll = Vec2::new(scroll.y, scroll.x);
                                }
                                if self.lock_x {
                                    scroll.x = 0.0;
                                }
                                if self.lock_y {
                                    scroll.y = 0.0;
                                }
                                let delta_pos = self.scroll_speed * scroll;
                                plot_ui.translate_bounds(delta_pos);
                            }
                        }
                        if plot_ui.response().hovered() && pointer_down {
                            let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                            if self.lock_x {
                                pointer_translate.x = 0.0;
                            }
                            if self.lock_y {
                                pointer_translate.y = 0.0;
                            }
                            plot_ui.translate_bounds(pointer_translate);
                        }

                        if self.gpu_temperature.len() < 2 {
                            plot_ui.line(Line::new(PlotPoints::default()).name("GPU"));

                            plot_ui.line(Line::new(PlotPoints::default()).name("CPU"));
                        } else {
                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&self.gpu_temperature[..])).name("GPU").width(5.0));

                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&self.cpu_temperature[..])).name("CPU").width(5.0));
                        }
                    });
            });

            if self.inner_timer == None {
                self.inner_timer = Some(Instant::now());
            }

            if self.inner_timer.unwrap().elapsed() >= self.time_between_update {
                let update_time = Instant::now();

                self.gpu_temperature.push(get_gpu_current_celsius_temperature_nvml(&mut self.nvml));

                println!("gpu update_time get {}", update_time.elapsed().as_millis());

                let update_time = Instant::now();

                self.cpu_temperature.push(get_cpu_current_celsius_temperature_using_wmi(&self.wmi_server)[0]);

                println!("cpu update_time get {}", update_time.elapsed().as_millis());

                self.inner_timer = None;
            }
    }
}

pub fn run_single_threaded_implementation() -> Result<(), eframe::Error> {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
    
        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        ).unwrap();
    }

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Resource monitor",
        options,
        Box::new(|_cc| Box::<PlotExample>::default()),
    )
}