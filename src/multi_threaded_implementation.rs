use std::time::{Duration, Instant};

use eframe::egui::{self, Event, Vec2};
use egui_plot::{Legend, Line, PlotPoints};
use crate::cpu_temperature::{get_cpu_current_celsius_temperature};
use crate::gpu_temperature::{get_gpu_current_celsius_temperature, get_gpu_current_celsius_temperature_nvml};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use windows::{
    Win32::System::Com::*,
    //Win32::System::Wmi::*,
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
    lock_x: AtomicBool,
    lock_y: AtomicBool,
    ctrl_to_zoom: AtomicBool,
    shift_to_horizontal: AtomicBool,
    zoom_speed: Arc<f32>,
    scroll_speed: Arc<f32>,
    time_between_update: Arc<Duration>,
    inner_timer: Arc<RwLock<Option<Instant>>>,
    gpu_temperature: Arc<RwLock<Vec<f32>>>,
    cpu_temperature: Arc<RwLock<Vec<f32>>>,
    //wmi_server: Arc<IWbemServices>,
    nvml: Arc<RwLock<Nvml>>,
    is_program_finished_working: Arc<RwLock<bool>>
}

impl Default for PlotExample {
    fn default() -> Self {
        //unsafe {
            /*
                ініціалізую інтерфейс IWbemServices, він використовується 
                клієнтами та постачальниками для доступу до служб WMI
            */
            /*
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).unwrap();
            let wmi_server = locator.ConnectServer(&windows::core::BSTR::from("root\\cimv2"), None, None, None, 0, None, None).unwrap();
            */

            // ініціалізую nvml
            let nvml = Nvml::init().unwrap();

            Self {
                lock_x: AtomicBool::new(true),
                lock_y: AtomicBool::new(true),
                ctrl_to_zoom: AtomicBool::new(false),
                shift_to_horizontal: AtomicBool::new(false),
                zoom_speed: Arc::new(1.0),
                scroll_speed: Arc::new(1.0),
                time_between_update: Arc::new(Duration::from_millis(2000)),
                inner_timer: Arc::new(RwLock::new(None)),
                gpu_temperature: Arc::new(RwLock::new(Vec::new())),
                cpu_temperature: Arc::new(RwLock::new(Vec::new())),
                nvml: Arc::new(RwLock::new(nvml)),
                //wmi_server: Arc::new(wmi_server),
                is_program_finished_working: Arc::new(RwLock::new(false))
            }
        //}
    }
}

impl eframe::App for PlotExample {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
            let gpu_temperature = self.gpu_temperature.read().unwrap();
            let cpu_temperature = self.cpu_temperature.read().unwrap();
            let lock_x = self.lock_x.load(Ordering::Relaxed);
            let lock_y = self.lock_y.load(Ordering::Relaxed);
            let ctrl_to_zoom = self.ctrl_to_zoom.load(Ordering::Relaxed);
            let shift_to_horizontal = self.shift_to_horizontal.load(Ordering::Relaxed);

            egui::SidePanel::left("options").show(&ctx, |ui| {
                ui.checkbox(&mut *self.lock_x.get_mut(), "Lock x axis").on_hover_text("Check to keep the X axis fixed, i.e., pan and zoom will only affect the Y axis");
                ui.checkbox(&mut *self.lock_y.get_mut(), "Lock y axis").on_hover_text("Check to keep the Y axis fixed, i.e., pan and zoom will only affect the X axis");
                ui.checkbox(&mut *self.ctrl_to_zoom.get_mut(), "Ctrl to zoom").on_hover_text("If unchecked, the behavior of the Ctrl key is inverted compared to the default controls\ni.e., scrolling the mouse without pressing any keys zooms the plot");
                ui.checkbox(&mut *self.shift_to_horizontal.get_mut(), "Shift for horizontal scroll").on_hover_text("If unchecked, the behavior of the shift key is inverted compared to the default controls\ni.e., hold to scroll vertically, release to scroll horizontally");
                
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
                            if modifiers.ctrl == ctrl_to_zoom {
                                scroll = Vec2::splat(scroll.x + scroll.y);
                                let mut zoom_factor = Vec2::from([
                                    (scroll.x * *self.zoom_speed / 10.0).exp(),
                                    (scroll.y * *self.zoom_speed / 10.0).exp(),
                                ]);
                                if lock_x {
                                    zoom_factor.x = 1.0;
                                }
                                if lock_y {
                                    zoom_factor.y = 1.0;
                                }
                                plot_ui.zoom_bounds_around_hovered(zoom_factor);
                            } else {
                                if modifiers.shift == shift_to_horizontal {
                                    scroll = Vec2::new(scroll.y, scroll.x);
                                }
                                if lock_x {
                                    scroll.x = 0.0;
                                }
                                if lock_y {
                                    scroll.y = 0.0;
                                }
                                let delta_pos = *self.scroll_speed * scroll;
                                plot_ui.translate_bounds(delta_pos);
                            }
                        }
                        if plot_ui.response().hovered() && pointer_down {
                            let mut pointer_translate = -plot_ui.pointer_coordinate_drag_delta();
                            if lock_x {
                                pointer_translate.x = 0.0;
                            }
                            if lock_y {
                                pointer_translate.y = 0.0;
                            }
                            plot_ui.translate_bounds(pointer_translate);
                        }

                        if gpu_temperature.len() < 2 {
                            plot_ui.line(Line::new(PlotPoints::default()).name("GPU").width(5.0));

                            plot_ui.line(Line::new(PlotPoints::default()).name("CPU").width(5.0));
                        } else {
                            //let gpu_temperature_points = PlotPoints::from_ys_f32(&gpu_temperature[..]);

                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&gpu_temperature[..])).name("GPU").width(5.0));

                            //let cpu_temperature_points = PlotPoints::from_ys_f32(&cpu_temperature[..]);

                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&cpu_temperature[..])).name("CPU").width(5.0));
                        }
                    });
            });
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        *self.is_program_finished_working.write().unwrap() = true;
    }
}

pub fn run_multi_threaded_implementation() -> Result<(), eframe::Error> {
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

    let plot = Box::<PlotExample>::default();

    let time_between_update = plot.time_between_update.clone();
    let inner_timer = plot.inner_timer.clone();
    let gpu_temperature = plot.gpu_temperature.clone();
    let cpu_temperature = plot.cpu_temperature.clone();
    let is_program_finished_working = plot.is_program_finished_working.clone();
    let nvml = plot.nvml.clone();

    let thread = std::thread::spawn(move || {
        loop {
            let read_inner_timer = *inner_timer.read().unwrap();

            if read_inner_timer == None {
                *inner_timer.write().unwrap() = Some(Instant::now());
            } else if read_inner_timer.unwrap().elapsed() >= *time_between_update {
                let mut gpu_temperature_inner = Vec::new();
                let mut cpu_temperature_inner = Vec::new();

                let update_time = Instant::now();
                
                gpu_temperature_inner.push(get_gpu_current_celsius_temperature_nvml(&mut nvml.write().unwrap()));

                println!("gpu update_time get {}", update_time.elapsed().as_millis());

                let update_time = Instant::now();

                cpu_temperature_inner.push(get_cpu_current_celsius_temperature());

                println!("cpu update_time get {}", update_time.elapsed().as_millis());

                gpu_temperature.write().unwrap().append(&mut gpu_temperature_inner);
                cpu_temperature.write().unwrap().append(&mut cpu_temperature_inner);

                *inner_timer.write().unwrap() = None;
            }

            if *is_program_finished_working.read().unwrap() {
                break;
            }
        }
    });

    let options = eframe::NativeOptions::default();

    let eframe = eframe::run_native(
        "Resource monitor",
        options,
        Box::new(|_cc| plot),
    );

    let _result_thread = thread.join();

    eframe
}