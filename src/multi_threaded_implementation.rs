use std::time::{Duration, Instant};

use eframe::egui::{self, Event, Vec2};
use egui_plot::{Legend, Line, PlotPoints};
use crate::cpu_temperature::{get_cpu_current_celsius_temperature, get_cpu_name};
use crate::gpu_temperature::{get_gpu_current_celsius_temperature, get_gpu_current_celsius_temperature_nvml, get_gpu_name_nvml};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use windows::{
    Win32::System::Com::*,
    Win32::System::Wmi::*,
};

use nvml_wrapper::Nvml;

struct PlotExample {
    delay_between_temperature_requests: Arc<RwLock<u64>>,
    inner_timer: Arc<RwLock<Option<Instant>>>,
    is_display_gpu_temperature: Arc<RwLock<bool>>,
    is_display_cpu_temperature: Arc<RwLock<bool>>,
    gpu_temperature: Arc<RwLock<Vec<f32>>>,
    cpu_temperature: Arc<RwLock<Vec<f32>>>,
    cpu_name: String,
    gpu_name: String,
    amount_of_stored_data: Arc<RwLock<u16>>,
    wmi_server: Arc<IWbemServices>,
    nvml: Arc<RwLock<Nvml>>,
    delay_between_updates: Arc<RwLock<u64>>,
    is_program_finished_working: Arc<RwLock<bool>>
}

impl Default for PlotExample {
    fn default() -> Self {
        let locator: IWbemLocator;
        let wmi_server;

        unsafe {
            // ініціалізую інтерфейс IWbemServices, він використовується для доступу до служб WMI
            locator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).unwrap();
            wmi_server = locator.ConnectServer(&windows::core::BSTR::from("root\\cimv2"), None, None, None, 0, None, None).unwrap();
        }
        // ініціалізую nvml
        let mut nvml = Nvml::init().unwrap();

        let cpu_name = get_cpu_name(&wmi_server);

        let gpu_name = get_gpu_name_nvml(&mut nvml);

        Self {
            delay_between_temperature_requests: Arc::new(RwLock::new(500)),
            inner_timer: Arc::new(RwLock::new(None)),
            is_display_gpu_temperature: Arc::new(RwLock::new(true)),
            is_display_cpu_temperature: Arc::new(RwLock::new(true)),
            gpu_temperature: Arc::new(RwLock::new(Vec::new())),
            cpu_temperature: Arc::new(RwLock::new(Vec::new())),
            cpu_name,
            gpu_name,
            amount_of_stored_data: Arc::new(RwLock::new(1200)),
            nvml: Arc::new(RwLock::new(nvml)),
            delay_between_updates: Arc::new(RwLock::new(16)),
            wmi_server: Arc::new(wmi_server),
            is_program_finished_working: Arc::new(RwLock::new(false))
        }
    }
    
}

impl eframe::App for PlotExample {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let gpu_temperature = self.gpu_temperature.read().unwrap();
        let cpu_temperature = self.cpu_temperature.read().unwrap();

        let is_display_gpu_temperature_read = *self.is_display_gpu_temperature.read().unwrap();
        let is_display_cpu_temperature_read = *self.is_display_cpu_temperature.read().unwrap();

        egui::SidePanel::left("options").show(&ctx, |ui| {
            ui.heading("Відображення");
            ui.add_space(10.0);

            ui.checkbox(&mut self.is_display_gpu_temperature.write().unwrap(), "Відображати температуру відеокарти");
            ui.checkbox(&mut self.is_display_cpu_temperature.write().unwrap(), "Відображати температуру процесора");
            ui.add_space(10.0);

            ui.add(egui::Separator::default());
            ui.add_space(10.0);

            ui.heading("Оптимізація");
            ui.add_space(10.0);

            ui.collapsing("Детальніше про оптимізацію", |ui| {
                ui.label(
                    "Ця вкладка надає можливість самостійно оптимізувати програму."
                );
                ui.add_space(5.0);
                ui.label("Наприклад:");
                ui.label("- при використанні програми в фоновому режимі, щоб заощадити ресурси системи, програму можна попередньо налавштувати, збільшивши затримку між оновленням рендеру, та збільшивши затримку між запитами на отримання температури, та після цього згорнути програму.");
            });
            ui.add_space(10.0);

            ui.label("Затримка між запитами на отримання температури (ms)").on_hover_text("регулюємо частуту отримання данних про температуру, чим більше значення, тим більша затримка до отримання данних.");
            ui.add(egui::Slider::new(&mut *self.delay_between_temperature_requests.write().unwrap(), 0..=3000)).on_hover_text("регулюємо частуту отримання данних про температуру, чим більше значення, тим більша затримка до отримання данних.");
            ui.add_space(10.0);

            ui.label("Затримка між викликами рендеру (ms)").on_hover_text("регулюємо затримку між оновленнями рендеру кожного кадру, простіше кажучи - дозволяє збільшувати, або зменшувати обмеження fps. Чим більше значення тим більша затримка, та нижчий fps, та відповідно меньше навантаження на систему.");
            ui.add(egui::Slider::new(&mut *self.delay_between_updates.write().unwrap(), 1..=100)).on_hover_text("регулюємо затримку між оновленнями рендеру кожного кадру, простіше кажучи - дозволяє збільшувати, або зменшувати обмеження fps. Чим більше значення тим більша затримка, та нижчий fps, та відповідно меньше навантаження на систему.");
            ui.add_space(10.0);

            ui.label("Кількість відображених даних про температуру").on_hover_text("регулюємо кількість елментів графіка відображених на екрані. Після накопичення вказаного значення найстаріші значеня починають по одному видалятися, як тільки надходять нові данні. Чим більше значення, тим більша кількість елментів буде збережена, та відобоажена на екрані (за певний проміжок часу).");
            ui.add(egui::Slider::new(&mut *self.amount_of_stored_data.write().unwrap(), 10..=1200)).on_hover_text("регулюємо кількість елментів графіка відображених на екрані. Після накопичення вказаного значення найстаріші значеня починають по одному видалятися, як тільки надходять нові данні. Чим більше значення, тим більша кількість елментів буде збережена, та відобоажена на екрані (за певний проміжок часу).");
            
        });

        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.heading("Графік температур процесора та відеокарти");
            ui.add_space(10.0);

            ui.label(format!("Процесор: {}", &self.cpu_name));
            ui.add_space(10.0);

            ui.label(format!("Відеокарта: {}", &self.gpu_name));
            ui.add_space(10.0);

            egui_plot::Plot::new("resource_monitor")
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .show_axes(egui::Vec2b{x: false, y: true})
                .show_grid(egui::Vec2b{x: false, y: true})
                .legend(Legend::default()
                    .background_alpha(1.0)
                    .position(egui_plot::Corner::RightBottom)
                )
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{}: {:.*}°C", name, 1, value.y)
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
                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&gpu_temperature[..])).name("GPU").width(5.0).color(egui::Color32::GREEN));
                        }

                        if is_display_cpu_temperature_read {
                            plot_ui.line(Line::new(PlotPoints::from_ys_f32(&cpu_temperature[..])).name("CPU").width(5.0).color(egui::Color32::RED));
                        }
                    }
                });
        });

        std::thread::sleep(Duration::from_millis(*self.delay_between_updates.read().unwrap())); // обмежую "fps" програми, щоб заощадити ресурси комп'ютера

        ctx.request_repaint();
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

    let delay_between_temperature_requests = plot.delay_between_temperature_requests.clone();
    let inner_timer = plot.inner_timer.clone();
    let gpu_temperature = plot.gpu_temperature.clone();
    let cpu_temperature = plot.cpu_temperature.clone();
    let is_program_finished_working = plot.is_program_finished_working.clone();
    let nvml = plot.nvml.clone();
    let is_display_gpu_temperature = plot.is_display_gpu_temperature.clone();
    let is_display_cpu_temperature = plot.is_display_cpu_temperature.clone();
    let amount_of_stored_data = plot.amount_of_stored_data.clone();

    let thread = std::thread::spawn(move || {
        loop {
            let read_inner_timer = *inner_timer.read().unwrap();

            if read_inner_timer == None {
                *inner_timer.write().unwrap() = Some(Instant::now());
            } else if read_inner_timer.unwrap().elapsed() >= Duration::from_millis(*delay_between_temperature_requests.read().unwrap()) {
                let mut gpu_temperature_inner = 0.0;
                let mut cpu_temperature_inner = 0.0;

                let is_display_gpu_temperature_read = *is_display_gpu_temperature.read().unwrap();
                let is_display_cpu_temperature_read = *is_display_cpu_temperature.read().unwrap();
                let amount_of_stored_data_read = *amount_of_stored_data.read().unwrap();

                if is_display_gpu_temperature_read {                    
                    gpu_temperature_inner = get_gpu_current_celsius_temperature_nvml(&mut nvml.write().unwrap());
                }

                if is_display_cpu_temperature_read {
                    cpu_temperature_inner = get_cpu_current_celsius_temperature();
                }

                gpu_temperature.write().unwrap().push(gpu_temperature_inner);

                if gpu_temperature.read().unwrap().len() > amount_of_stored_data_read as usize {
                    gpu_temperature.write().unwrap().remove(0);
                }

                cpu_temperature.write().unwrap().push(cpu_temperature_inner);

                if cpu_temperature.read().unwrap().len() > amount_of_stored_data_read as usize {
                    cpu_temperature.write().unwrap().remove(0);
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