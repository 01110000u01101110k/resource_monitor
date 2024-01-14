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

struct PlotExample {
    time_between_update: Duration,
    inner_timer: Option<Instant>,
    is_display_gpu_temperature: bool,
    is_display_cpu_temperature: bool,
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
                time_between_update: Duration::from_millis(500),
                inner_timer: None,
                is_display_gpu_temperature: true,
                is_display_cpu_temperature: true,
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
                ui.checkbox(&mut self.is_display_gpu_temperature, "відображати температуру відеокарти").on_hover_text("");
                ui.checkbox(&mut self.is_display_cpu_temperature, "відображати температуру процесора").on_hover_text("");
            });
            egui::CentralPanel::default().show(&ctx, |ui| {
                ui.label("графік температур процесора та відеокарти");

                egui_plot::Plot::new("resource_monitor")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        if self.gpu_temperature.len() < 2 {
                            plot_ui.line(Line::new(PlotPoints::default()).name("GPU"));

                            plot_ui.line(Line::new(PlotPoints::default()).name("CPU"));
                        } else {
                            if self.is_display_gpu_temperature {
                                plot_ui.line(Line::new(PlotPoints::from_ys_f32(&self.gpu_temperature[..])).name("GPU").width(5.0));
                            }

                            if self.is_display_cpu_temperature {
                                plot_ui.line(Line::new(PlotPoints::from_ys_f32(&self.cpu_temperature[..])).name("CPU").width(5.0));
                            }
                        }
                    });
            });

            if self.inner_timer == None {
                self.inner_timer = Some(Instant::now());
            }

            if self.inner_timer.unwrap().elapsed() >= self.time_between_update {
                //let update_time = Instant::now();

                if self.is_display_gpu_temperature {
                    self.gpu_temperature.push(get_gpu_current_celsius_temperature_nvml(&mut self.nvml));

                    if self.gpu_temperature.len() > 120 {
                        self.gpu_temperature.remove(0);
                        self.gpu_temperature.remove(0);
                    }
                }

                //println!("gpu update_time get {}", update_time.elapsed().as_millis());

                //let update_time = Instant::now();

                if self.is_display_cpu_temperature {
                    self.cpu_temperature.push(get_cpu_current_celsius_temperature_using_wmi(&self.wmi_server)[0]);

                    if self.cpu_temperature.len() > 120 {
                        self.cpu_temperature.remove(0);
                        self.cpu_temperature.remove(0);
                    }
                }

                //println!("cpu update_time get {}", update_time.elapsed().as_millis());

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