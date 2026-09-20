use std::collections::HashMap;
use std::collections::HashSet;

use crate::models::TraitModel;
use crate::tools::macros::tr;
use crate::tools::utils;
use colored::Colorize;
use human_sort::sort;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct FlutterAvailableModel {
    pub url: String,
    pub version_full: String,
}

impl TraitModel for FlutterAvailableModel {
    fn get_id(&self) -> String {
        format!("{:x}", md5::compute(self.version_full.as_bytes()))
    }

    fn get_key(&self) -> String {
        self.version_full.clone()
    }

    fn print(&self) {
        println!(
            "{}",
            tr!(
                "Flutter SDK: {}\nСсылка: {}",
                self.version_full.bold().white(),
                self.url.to_string().bright_blue(),
            )
        );
    }
}

impl FlutterAvailableModel {
    pub fn search() -> Vec<FlutterAvailableModel> {
        match Self::search_full() {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Ошибка при получении списка версий Flutter SDK: {}", e);
                vec![]
            }
        }
    }

    pub fn search_filter<T: Fn(&FlutterAvailableModel) -> bool>(filter: T) -> Vec<FlutterAvailableModel> {
        Self::search().into_iter().filter(filter).collect()
    }

    fn search_full() -> Result<Vec<FlutterAvailableModel>, Box<dyn std::error::Error>> {
        let url_files = utils::get_repo_url_flutter_sdk();

        // Универсальный Regex для извлечения версии (например, "3.41.4" или "3.41.4-beta.1")
        let version_regex = Regex::new(r"flutter_aurora(?:_[^_]+)?_([^_]+)\.tar\.gz")?;

        let mut version_urls: HashMap<String, Vec<String>> = HashMap::new();
        let mut versions_set: HashSet<String> = HashSet::new();

        for url in url_files {
            // ПЕРЕВЕДЕНО В НИЖНИЙ РЕГИСТР ДЛЯ НАДЁЖНОГО ПОИСКА
            let lower_url = url.to_lowercase();

            // ОТСЕКАЕМ mac И beta ОДНОВРЕМЕННО
            if lower_url.contains("mac") || lower_url.contains("beta") {
                continue;
            }

            // Извлекаем имя файла из полного URL
            let file_name = url.split('/').last().unwrap_or("");

            // Извлекаем версию с помощью регулярного выражения
            if let Some(caps) = version_regex.captures(file_name) {
                let version_full = caps.get(1).unwrap().as_str().to_string();

                version_urls
                    .entry(version_full.clone())
                    .or_insert_with(Vec::new)
                    .push(url);

                versions_set.insert(version_full);
            }
        }

        // Сортируем версии с помощью human_sort (от новых к старым)
        let mut versions: Vec<&str> = versions_set.iter().map(|s| s.as_str()).collect();
        sort(&mut versions);
        versions.reverse();

        // Формируем итоговый вектор моделей без дубликатов
        let mut models: Vec<FlutterAvailableModel> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for version_full in versions {
            if let Some(urls) = version_urls.get(version_full) {
                for url in urls {
                    let model = FlutterAvailableModel {
                        url: url.clone(),
                        version_full: version_full.to_string(),
                    };

                    let id = model.get_id();
                    if !seen_ids.contains(&id) {
                        seen_ids.insert(id);
                        models.push(model);
                    }
                }
            }
        }

        Ok(models)
    }
}
