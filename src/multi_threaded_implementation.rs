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

struct PlotExample {
    time_between_update: Arc<Duration>,
    inner_timer: Arc<RwLock<Option<Instant>>>,
    is_display_gpu_temperature: Arc<RwLock<bool>>,
    is_display_cpu_temperature: Arc<RwLock<bool>>,
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
                time_between_update: Arc::new(Duration::from_millis(2000)),
                inner_timer: Arc::new(RwLock::new(None)),
                is_display_gpu_temperature: Arc::new(RwLock::new(true)),
                is_display_cpu_temperature: Arc::new(RwLock::new(true)),
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

        let is_display_gpu_temperature_read = *self.is_display_gpu_temperature.read().unwrap();
        let is_display_cpu_temperature_read = *self.is_display_cpu_temperature.read().unwrap();

        egui::SidePanel::left("options").show(&ctx, |ui| {
            ui.checkbox(&mut *self.is_display_gpu_temperature.write().unwrap(), "відображати температуру відеокарти").on_hover_text("");
            ui.checkbox(&mut *self.is_display_cpu_temperature.write().unwrap(), "відображати температуру процесора").on_hover_text("");
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("графік температур процесора та відеокарти");

            egui_plot::Plot::new("resource_monitor")
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .legend(Legend::default())
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("Температура {}: {:.*}°C", name, 1, value.y)
                    } else {
                        "".to_owned()
                    }
                })
                .show(ui, |plot_ui| {
                    if gpu_temperature.len() < 2 {
                        plot_ui.line(Line::new(PlotPoints::default()).name("GPU").width(5.0));

                        plot_ui.line(Line::new(PlotPoints::default()).name("CPU").width(5.0));
                    } else {
                        if is_display_gpu_temperature_read {
                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&gpu_temperature[..])).name("GPU").width(5.0));
                        }

                        if is_display_cpu_temperature_read {
                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&cpu_temperature[..])).name("CPU").width(5.0));
                        }
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
    let is_display_gpu_temperature = plot.is_display_gpu_temperature.clone();
    let is_display_cpu_temperature = plot.is_display_cpu_temperature.clone();

    let thread = std::thread::spawn(move || {
        loop {
            let read_inner_timer = *inner_timer.read().unwrap();

            if read_inner_timer == None {
                *inner_timer.write().unwrap() = Some(Instant::now());
            } else if read_inner_timer.unwrap().elapsed() >= *time_between_update {
                let mut gpu_temperature_inner = Vec::new();
                let mut cpu_temperature_inner = Vec::new();

                let is_display_gpu_temperature_read = *is_display_gpu_temperature.read().unwrap();
                let is_display_cpu_temperature_read = *is_display_cpu_temperature.read().unwrap();

                if is_display_gpu_temperature_read {
                    let update_time = Instant::now();
                    
                    gpu_temperature_inner.push(get_gpu_current_celsius_temperature_nvml(&mut nvml.write().unwrap()));

                    println!("gpu update_time get {}", update_time.elapsed().as_millis());
                }

                if is_display_cpu_temperature_read {
                    let update_time = Instant::now();

                    cpu_temperature_inner.push(get_cpu_current_celsius_temperature());

                    println!("cpu update_time get {}", update_time.elapsed().as_millis());
                }

                if is_display_gpu_temperature_read {
                    gpu_temperature.write().unwrap().append(&mut gpu_temperature_inner);

                    if gpu_temperature.read().unwrap().len() > 120 {
                        gpu_temperature.write().unwrap().remove(0);
                        gpu_temperature.write().unwrap().remove(0);
                    }
                }

                if is_display_cpu_temperature_read {
                    cpu_temperature.write().unwrap().append(&mut cpu_temperature_inner);

                    if cpu_temperature.read().unwrap().len() > 120 {
                        cpu_temperature.write().unwrap().remove(0);
                        cpu_temperature.write().unwrap().remove(0);
                    }
                }

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