#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{self, Slider};
use rand::Rng;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Card Collection Calculator",
        options,
        Box::new(|_| Box::<MyApp>::default()),
    )
}

#[derive(Debug, Clone, Copy)]
struct Card {
    id: u32,
    need: bool,
    prob: u32,
}

struct MyApp {
    prob: String,
    selects: String,
    num: u32,
    results: Arc<Mutex<Vec<u32>>>,
    duration: Option<Duration>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            prob: "5, 10, 15, 20, 25".to_string(),
            selects: "1,2,3,4,5".to_string(),
            num: 10000,
            results: Arc::new(Mutex::new(vec![])),
            duration: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let prob_label = ui.label("Probabilities: ");
                ui.text_edit_singleline(&mut self.prob)
                    .labelled_by(prob_label.id);
            });
            ui.horizontal(|ui| {
                let selects_label = ui.label("Selects: ");
                ui.text_edit_singleline(&mut self.selects)
                    .labelled_by(selects_label.id);
            });
            ui.horizontal(|ui| {
                let num_label = ui.label("Number: ");
                ui.add(Slider::new(&mut self.num, 1..=100000).text(""))
                    .labelled_by(num_label.id);
            });
            if ui.button("Calculate").clicked() {
                let start = Instant::now();

                let selects: Vec<u32> = self
                    .selects
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.trim().parse().unwrap())
                    .collect();

                let cards: Vec<Card> = self
                    .prob
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .enumerate()
                    .map(|(index, s)| Card {
                        id: index as u32 + 1,
                        need: selects.contains(&(index as u32 + 1)),
                        prob: s.trim().parse().unwrap(),
                    })
                    .collect();
                // println!("{:?}", self.cards);

                self.results.lock().unwrap().clear();
                let num = self.num;
                let prob_sum = cards.iter().map(|c| c.prob).sum();
                let need_count = cards.iter().filter(|c| c.need).count();
                let handles = (0..10).map(|_| {
                    let cards_clone = cards.clone();
                    let results_clone = Arc::clone(&self.results);
                    thread::spawn(move || {
                        for _ in 0..num / 10 {
                            let mut gets = HashMap::new();
                            for n in 1.. {
                                let mut rand = rand::thread_rng().gen_range(1..=prob_sum);
                                for card in cards_clone.iter() {
                                    if rand <= card.prob {
                                        if card.need {
                                            gets.insert(card.id, n);
                                        }
                                        break;
                                    } else {
                                        rand -= card.prob;
                                    }
                                }
                                if gets.len() == need_count {
                                    results_clone.lock().unwrap().push(n);
                                    break;
                                }
                            }
                        }
                    })
                });

                for handle in handles {
                    handle.join().unwrap();
                }

                self.duration = Some(start.elapsed());
                // println!("{:?}", self.duration);
            }

            ui.separator();

            let results = self.results.lock().unwrap();
            let play_sum: u32 = results.iter().sum();
            ui.horizontal(|ui| {
                ui.label("Avg: ");
                ui.label((play_sum as f32 / results.len() as f32).to_string());
            });
            ui.horizontal(|ui| {
                ui.label("Duration: ");
                if let Some(duration) = self.duration {
                    ui.label(duration.as_millis().to_string() + "ms");
                }
            });
        });
    }
}
