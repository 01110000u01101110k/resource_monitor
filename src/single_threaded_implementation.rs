use std::time::{Duration, Instant};

use eframe::egui::{self, Event, Vec2};
use egui_plot::{Legend, Line, PlotPoints};
use crate::cpu_temperature::{get_cpu_current_celsius_temperature_using_wmi, get_cpu_name};
use crate::gpu_temperature::{get_gpu_current_celsius_temperature, get_gpu_current_celsius_temperature_nvml, get_gpu_name_nvml};

use windows::{
    Win32::System::Com::*,
    Win32::System::Wmi::*,
};

use nvml_wrapper::Nvml;

struct PlotExample {
    delay_between_temperature_requests: u16,
    inner_timer: Option<Instant>,
    is_display_gpu_temperature: bool,
    is_display_cpu_temperature: bool,
    gpu_temperature: Vec<f32>,
    cpu_temperature: Vec<f32>,
    cpu_name: String,
    gpu_name: String,
    amount_of_stored_data: u16,
    wmi_server: IWbemServices,
    nvml: Nvml,
    delay_between_updates: u8
}

impl Default for PlotExample {
    fn default() -> Self {
        unsafe {
            // ініціалізую інтерфейс IWbemServices, він використовується для доступу до служб WMI
            let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).unwrap();
            let wmi_server = locator.ConnectServer(&windows::core::BSTR::from("root\\cimv2"), None, None, None, 0, None, None).unwrap();

            // ініціалізую nvml
            let mut nvml = Nvml::init().unwrap();

            let cpu_name = get_cpu_name(&wmi_server);

            let gpu_name = get_gpu_name_nvml(&mut nvml);

            Self {
                delay_between_temperature_requests: 500,
                inner_timer: None,
                is_display_gpu_temperature: true,
                is_display_cpu_temperature: true,
                gpu_temperature: Vec::new(),
                cpu_temperature: Vec::new(),
                cpu_name,
                gpu_name,
                amount_of_stored_data: 600,
                wmi_server,
                nvml,
                delay_between_updates: 12
            }
        }
    }
}

impl eframe::App for PlotExample {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("options").show(&ctx, |ui| {
            ui.heading("Відображення");
            ui.add_space(10.0);

            ui.checkbox(&mut self.is_display_gpu_temperature, "Відображати температуру відеокарти");
            ui.checkbox(&mut self.is_display_cpu_temperature, "Відображати температуру процесора");
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
                ui.label("- при використанні програми в фоновому режимі, щоб заощадити ресурси системи її можна попередньо налавштувати знизивши параметри швидкості оновлення рендеру, та збільшивши затримку між запитами на отримання температури, та після цього згорнути програму.");
            });
            ui.add_space(10.0);

            ui.label("Затримка між запитами на отримання температури (ms)");
            ui.add(egui::Slider::new(&mut self.delay_between_temperature_requests, 0..=3000)).on_hover_text("регулюємо частуту отримання данних про температуру, чим більше значення, тим більша затримка. Затримка рахується в мілісекундах - відповідно максимальне значення затримки 3 секунди.");
            ui.add_space(10.0);

            ui.label("Затримка між викликами рендеру (ms)");
            ui.add(egui::Slider::new(&mut self.delay_between_updates, 1..=100)).on_hover_text("регулюємо затримку між оновленнями рендеру кожного кадру, простіше кажучи - дозволяє збільшувати, або зменшувати обмеження fps. Чим більше значення тим більша затримка, та нижчий fps, та відповідно меньше навантаження на систему.");
            ui.add_space(10.0);

            ui.label("кількість відображених даних про температуру");
            ui.add(egui::Slider::new(&mut self.amount_of_stored_data, 10..=1200)).on_hover_text("регулюємо кількість елментів графіка відображених на екрані. Після накопичення вказаного значення найстаріші значеня починають по одному видалятися, як тільки надходять нові данні. Чим більше значення, тим більша кількість елментів буде збережена, та відобоажена на екрані (за певний проміжок часу).");
            
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
                    .position(egui_plot::Corner::LeftTop)
                )
                .label_formatter(|name, value| {
                    if !name.is_empty() {
                        format!("{}: {:.*}°C", name, 1, value.y)
                    } else {
                        "".to_owned()
                    }
                })
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
        } else if self.inner_timer.unwrap().elapsed() >= Duration::from_millis(self.delay_between_temperature_requests as u64) {
            //let update_time = Instant::now();

            if self.is_display_gpu_temperature {
                self.gpu_temperature.push(get_gpu_current_celsius_temperature_nvml(&mut self.nvml));

                if self.gpu_temperature.len() > self.amount_of_stored_data as usize {
                    self.gpu_temperature.remove(0);
                    self.gpu_temperature.remove(0);
                }
            }

            //println!("gpu update_time get {}", update_time.elapsed().as_millis());

            //let update_time = Instant::now();

            if self.is_display_cpu_temperature {
                self.cpu_temperature.push(get_cpu_current_celsius_temperature_using_wmi(&self.wmi_server)[0]);

                if self.cpu_temperature.len() > self.amount_of_stored_data as usize {
                    self.cpu_temperature.remove(0);
                    self.cpu_temperature.remove(0);
                }
            }

            //println!("cpu update_time get {}", update_time.elapsed().as_millis());

            self.inner_timer = None;
        }

        std::thread::sleep(Duration::from_millis(self.delay_between_updates as u64)); // обмежую "fps" програми, щоб заощадити ресурси комп'ютера

        ctx.request_repaint();
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

    let mut options = eframe::NativeOptions::default();

    options.centered = true;
    //options.viewport.window_level = Some(egui::viewport::WindowLevel::AlwaysOnTop);

    eframe::run_native(
        "Resource monitor",
        options,
        Box::new(|_cc| Box::<PlotExample>::default()),
    )
}