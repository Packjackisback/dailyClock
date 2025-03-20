use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::sync::Arc;
use serde_json::{self, Value};
use eframe::{App, Frame, CreationContext, egui};
use egui::{Ui, ScrollArea, RichText, Color32, Layout, Align};
use chrono::{DateTime, Local, Utc, Timelike, Duration};
use serde::Deserialize;

mod models;
mod routines;
use routines::get_today_schedule;
use models::{Exercise, Workout, Conditioning, Cardio};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1920 as f32, 1080 as f32]),
        ..Default::default()
    };

    eframe::run_native(
        "Workout Tracker",
        options,
        Box::new(|cc| {
            let mut font_data = Vec::new();
            if let Ok(mut file) = fs::File::open("src/InconsolataNerdFontMono-Regular.ttf") {
                file.read_to_end(&mut font_data).unwrap();
            } else {
                eprintln!("Failed to open font file.");
            }

            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "NotoSans-Regular".to_owned(),
                Arc::from(egui::FontData::from_owned(font_data)),
            );
            fonts.families.insert(
                egui::FontFamily::Proportional,
                vec!["NotoSans-Regular".to_owned()],
            );
            fonts.families.insert(
                egui::FontFamily::Monospace,
                vec!["NotoSans-Regular".to_owned()], 
            );

            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(WorkoutApp::new(cc)))
        }),
    )
}

struct NewsHeadline {
    title: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct HourlyForecast {
    dt: i64,
    main: Main,
    weather: Vec<Weather>,
}
#[derive(Deserialize, Debug)]
struct HourlyForecastResponse {
    list: Vec<HourlyForecast>,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f32,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
    icon: String, 
}

struct WeatherData {
    temperature: String,
    conditions: String,
    hourly_forecast: Vec<(String, String, String, String)>,
}

struct WorkoutApp {
    workouts: Vec<Workout>,
    conditioning: Vec<Conditioning>,
    today_schedule: Option<routines::DaySchedule>,
    selected_workout: Option<usize>,
    selected_conditioning: Option<usize>,
    display_mode: DisplayMode,
    weather_data: Option<WeatherData>, 
    weather_api_key: String,
    news_headlines: Vec<NewsHeadline>, 
    greeting_message: String,
    last_view_change_time: Option<DateTime<Local>>,
    last_greeting_update_time: Option<DateTime<Local>>,
    last_weather_update_time: Option<DateTime<Local>>
}

#[derive(PartialEq, Clone, Copy)]
enum DisplayMode {
    Greeting, 
    Workout,
    Weather,
    News,
}

impl WorkoutApp {
    fn new(_cc: &CreationContext) -> Self {
        let json_data = fs::read_to_string("src/workouts.json").unwrap_or_default();
        let raw_data: HashMap<String, Vec<Value>> = serde_json::from_str(&json_data).unwrap_or_default();

        let mut workouts = Vec::new();
        workouts.push(create_workout("Upper A", raw_data.get("Upper A").unwrap_or(&Vec::new())));
        workouts.push(create_workout("Upper B", raw_data.get("Upper B").unwrap_or(&Vec::new())));
        workouts.push(create_workout("Lower A", raw_data.get("Lower A").unwrap_or(&Vec::new())));
        workouts.push(create_workout("Lower B", raw_data.get("Lower B").unwrap_or(&Vec::new())));

        let mut conditioning = Vec::new();
        conditioning.push(create_conditioning("Conditioning A", raw_data.get("Conditioning A").unwrap_or(&Vec::new())));
        conditioning.push(create_conditioning("Conditioning B", raw_data.get("Conditioning B").unwrap_or(&Vec::new())));

        let schedule_json = fs::read_to_string("src/schedule.json").unwrap_or_default();
        let today_schedule = get_today_schedule(&schedule_json);
        let last_greeting_update_time = None;
        let last_weather_update_time = None;
        let selected_workout = if let Some(schedule) = &today_schedule {
            match schedule.lifting.as_str() {
                "Upper A" => Some(0),
                "Upper B" => Some(1),
                "Lower A" => Some(2),
                "Lower B" => Some(3),
                _ => None,
            }
        } else {
            None
        };

        let news_headlines = vec![
            NewsHeadline {
                title: "Breaking News 1".to_string(),
                description: "Description of breaking news 1.".to_string(),
            },
            NewsHeadline {
                title: "Breaking News 2".to_string(),
                description: "Description of breaking news 2.".to_string(),
            },
        ];
        let weather_api_key = "3c28729c122f23510d7bfdadb12fc823".to_string();
        let mut app = WorkoutApp {
            workouts,
            conditioning,
            today_schedule,
            selected_workout,
            selected_conditioning: None,
            display_mode: DisplayMode::Greeting, 
            weather_data: None,
            weather_api_key,
            news_headlines,
            greeting_message: String::new(),
            last_view_change_time: None,
            last_greeting_update_time,
            last_weather_update_time
        };

        app.fetch_weather();
        app.update_greeting_message();
        app
    }

    fn update_greeting_message(&mut self) {
        let now = Local::now();
        let hour = now.hour();

        self.greeting_message = if hour >= 5 && hour < 12 {
            "Good Morning, Jackson".to_string()
        } else if hour >= 12 && hour < 19 {
            "Welcome home, Jackson".to_string()
        } else {
            "Good night, Jackson".to_string()
        };
    }
}

impl App for WorkoutApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let current_time = Local::now().format("%H:%M:%S").to_string();
        let now = Local::now();
        if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
            self.display_mode = DisplayMode::Workout;
            self.last_view_change_time = Some(Local::now());
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
            self.display_mode = DisplayMode::Weather;
            self.last_view_change_time = Some(Local::now());
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
            self.display_mode = DisplayMode::News;
            self.last_view_change_time = Some(Local::now());
        }

        if self.display_mode != DisplayMode::Greeting {
            if let Some(last_change) = self.last_view_change_time {
                if Local::now() - last_change >= Duration::seconds(20) {
                    self.display_mode = DisplayMode::Greeting;
                    self.last_view_change_time = None;
                }
            } else {
                self.last_view_change_time = Some(Local::now());
            }
        } else {
            self.last_view_change_time = None;
        }

        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(36.0, egui::FontFamily::Proportional),
        );
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.selectable_label(self.display_mode == DisplayMode::Workout, "Workout").clicked() {
                        self.display_mode = DisplayMode::Workout;
                        self.last_view_change_time = Some(Local::now());
                    }
                    if ui.selectable_label(self.display_mode == DisplayMode::Weather, "Weather").clicked() {
                        self.display_mode = DisplayMode::Weather;
                        self.last_view_change_time = Some(Local::now());
                    }
                    if ui.selectable_label(self.display_mode == DisplayMode::News, "News").clicked() {
                        self.display_mode = DisplayMode::News;
                        self.last_view_change_time = Some(Local::now());
                    }
                });

                ui.add_space(20.0);

                match self.display_mode {
                    DisplayMode::Greeting => self.show_greeting_display(ui, &current_time),
                    DisplayMode::Workout => self.show_workout_display(ui),
                    DisplayMode::Weather => self.show_weather_display(ui),
                    DisplayMode::News => self.show_news_display(ui),
                }
            });
        });
        if self.last_greeting_update_time.is_none() || now - self.last_greeting_update_time.unwrap() >= Duration::minutes(1) {
            self.update_greeting_message();
            self.last_greeting_update_time = Some(now);
        }
        if self.last_weather_update_time.is_none() || now - self.last_weather_update_time.unwrap() >= Duration::minutes(30) {
            self.fetch_weather();
            let json_data = fs::read_to_string("src/schedule.json").unwrap_or_default();
            self.today_schedule = get_today_schedule(&json_data);
            self.last_weather_update_time = Some(now);
        }

        ctx.request_repaint();
    }
}

impl WorkoutApp {
    fn show_greeting_display(&mut self, ui: &mut Ui, current_time: &str) {
        ui.add_space(50.0);
        ui.label(
            RichText::new(current_time)
                .heading()
                .size(100.0) 
                .strong(),
        );
        ui.add_space(20.0);
        ui.label(
            RichText::new(&self.greeting_message)
                .heading()
                .size(40.0)
                .strong(),
        );
        ui.add_space(50.0);


    }

    fn show_workout_display(&mut self, ui: &mut Ui) {
        if let Some(schedule) = &self.today_schedule {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Today's Schedule")
                    .heading()
                    .size(36.0)
                    .strong(),
            );

            ui.label(format!("Day: {}", schedule.day));
            ui.label(format!("Date: {}", schedule.date));
            ui.label(format!("Throwing: {}", schedule.throwing));
            ui.label(format!("Lifting: {}", schedule.lifting));
            ui.label(format!("Game: {}", schedule.game));

            if schedule.lifting.contains("OR") {
                ui.add_space(5.0);
                ui.label(
                    egui::RichText::new("Choose your workout:")
                        .size(32.0)
                        .strong(),
                );

                let options: Vec<&str> = schedule.lifting.split(" OR ").collect();
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        for option in options {
                            if ui.button(
                                egui::RichText::new(option).size(28.0)
                            ).clicked() {
                                self.selected_workout = match option {
                                    "Upper A" => Some(0),
                                    "Upper B" => Some(1),
                                    "Lower A" => Some(2),
                                    "Lower B" => Some(3),
                                    _ => None,
                                };
                            }
                        }
                    });
                });
            }
        } else {
            ui.label(
                egui::RichText::new("No schedule found for today.")
                    .size(28.0),
            );
        }

        ui.add_space(20.0);

        if let Some(idx) = self.selected_workout {
            if let Some(workout) = self.workouts.get(idx) {
                ui.label(
                    egui::RichText::new(format!("Workout: {}", workout.name))
                        .heading()
                        .size(36.0)
                        .strong(),
                );
                ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                        for exercise in &workout.exercises {
                            ui.horizontal(|ui| {
                                ui.label("•");
                                ui.label(
                                    egui::RichText::new(format!("{}", exercise.name))
                                        .size(28.0)
                                        .strong(),
                                );

                                ui.label(egui::RichText::new(": ").size(28.0));
                                ui.label(
                                    egui::RichText::new(format!("{}", exercise.sets))
                                        .size(28.0)
                                        .color(Color32::BLUE)
                                        .strong(),
                                );
                                ui.label(egui::RichText::new(" sets").size(28.0));

                                if let Some(reps) = exercise.reps {
                                    let each_side = if exercise.each { " each side" } else { "" };

                                    ui.label(egui::RichText::new(" - ").size(28.0));
                                    ui.label(
                                        egui::RichText::new(format!("{}", reps))
                                            .size(28.0)
                                            .color(Color32::RED)
                                            .strong(),
                                    );
                                    ui.label(egui::RichText::new(format!(" reps{}", each_side)).size(28.0));
                                } else if let Some(secs) = exercise.seconds {
                                    ui.label(egui::RichText::new(" - ").size(28.0));
                                    ui.label(
                                        egui::RichText::new(format!("{}", secs))
                                            .size(28.0)
                                            .color(Color32::GREEN)
                                            .strong(),
                                    );
                                    ui.label(egui::RichText::new(" seconds").size(28.0));
                                }
                            });
                        }
                    });
                });
            }
        }
    }

    fn show_weather_display(&mut self, ui: &mut Ui) {
        if let Some(weather) = &self.weather_data {
            ui.label(
                egui::RichText::new("Weather")
                    .heading()
                    .size(36.0)
                    .strong(),
            );
            ui.label(format!("Temperature: {}", weather.temperature));
            ui.label(format!("Conditions: {}", weather.conditions));
            ui.label(egui::RichText::new("Hourly Forecast:").strong());

            ScrollArea::horizontal().show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (time, temp, description, icon) in &weather.hourly_forecast {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(time).strong());
                            ui.label(temp);
                            ui.label(description);
                            let symbol = match icon.as_str() {
                                "01d" | "01n" => "\u{f185}", 
                                "02d" | "02n" => "\u{f186}", 
                                "03d" | "03n" | "04d" | "04n" => "\u{f0c2}",
                                "09d" | "09n" | "10d" | "10n" => "\u{f0e3}",
                                "11d" | "11n" => "\u{f0e7}",
                                "13d" | "13n" => "\u{f2dc}",
                                "50d" | "50n" => "\u{f760}",
                                _ => "",
                            };
                            ui.label(symbol);
                        });
                        ui.add_space(20.0);
                    }
                });
            });
        } else {
            ui.label(
                egui::RichText::new("Weather data not available.")
                    .size(28.0),
            );
        }
    }

    fn show_news_display(&mut self, ui: &mut Ui) {
        ui.label(
            egui::RichText::new("News Headlines")
                .heading()
                .size(36.0)
                .strong(),
        );
        ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
            for headline in &self.news_headlines {
                ui.label(
                    egui::RichText::new(format!("• {}", headline.title))
                        .size(28.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("  {}", headline.description))
                        .size(24.0),
                );
                ui.add_space(10.0);
            }
        });
    }
    fn fetch_weather(&mut self) {
        let api_key = &self.weather_api_key;
        let city = "Houston";
        let url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=imperial",
            city, api_key
        );

        let client = reqwest::blocking::Client::new();
        match client.get(&url).send() {
            Ok(response) => {
                match response.json::<HourlyForecastResponse>() {
                    Ok(weather_response) => {
                        let mut hourly_forecast = Vec::new();
                        let num_forecasts_to_take = weather_response.list.len().min(8);
                        for forecast in weather_response.list.iter().take(num_forecasts_to_take) {
                            let datetime = DateTime::<Utc>::from_utc(
                                chrono::NaiveDateTime::from_timestamp(forecast.dt, 0),
                                Utc,
                            );
                            let local_datetime: DateTime<Local> = DateTime::from(datetime);
                            let time_string = local_datetime.format("%H:%M").to_string();
                            let description = forecast.weather[0].description.clone();
                            let icon = forecast.weather[0].icon.clone();
                            hourly_forecast.push((
                                time_string,
                                format!("{:.1}°F", forecast.main.temp),
                                description,
                                icon,
                            ));
                        }

                        self.weather_data = Some(WeatherData {
                            temperature: format!("{:.1}°F", weather_response.list[0].main.temp),
                            conditions: weather_response.list[0].weather[0].description.clone(),
                            hourly_forecast,
                        });
                    }
                    Err(e) => eprintln!("Error parsing weather response: {}", e),
                }
            }
            Err(e) => eprintln!("Error fetching weather: {}", e),
        }
    }
}

fn create_workout(name: &str, raw_exercises: &[serde_json::Value]) -> Workout {
    let mut exercises = Vec::new();

    for raw_ex in raw_exercises {
        let exercise_name = raw_ex["exercise"].as_str().unwrap_or("").to_string();
        if exercise_name == "Warmup" {
            continue;
        }

        exercises.push(Exercise {
            name: exercise_name,
            sets: raw_ex["sets"].as_u64().unwrap_or(0) as u8,
            reps: raw_ex["reps"].as_u64().map(|r| r as u8),
            each: raw_ex["each"].as_bool().unwrap_or(false),
            seconds: raw_ex["seconds"].as_u64().map(|s| s as u8),
            weight: None,
        });
    }

    Workout {
        name: name.to_string(),
        exercises,
    }
}

fn create_conditioning(name: &str, raw_exercises: &[Value]) -> Conditioning {
    let choices = raw_exercises
        .iter()
        .map(|ex| Cardio {
            name: ex["exercise"].as_str().unwrap_or("").to_string(),
            description: ex["description"].as_str().unwrap_or("").to_string(),
            time: ex["seconds"].as_u64().map(|s| s as u8),
            rest: ex["rest"].as_u64().map(|r| r as u8),
            sets: ex["sets"].as_u64().unwrap_or(0) as u8,
        })
        .collect();

    Conditioning {
        name: name.to_string(),
        choices,
    }
}
