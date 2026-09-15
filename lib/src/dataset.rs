use std::{collections::HashMap, fs::read_to_string, io::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::rules::Rule;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Database {
    pub tags: Vec<Tag>,
    pub wallpapers: Vec<Wallpaper>,
    pub rules: Vec<Rule>,
}

impl Database {
    pub fn from_file(path: PathBuf) -> Result<Database, Error> {
        ron::from_str(read_to_string(path)?.as_str()).map_err(Error::other)
    }

    pub fn get_walls(
        &self,
        current_rules: &Vec<Rule>,
    ) -> Option<(Vec<Wallpaper>, Vec<u32>, Vec<Rule>)> {
        eprintln!("Getting new wallpaperset");
        let mut changed = false;
        let new_rules = self
            .rules
            .iter()
            .filter_map(|rule| {
                if rule.is_true() {
                    Some(rule.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<Rule>>();
        for rule in &new_rules {
            if !current_rules.contains(rule) {
                changed = true;
                break;
            }
        }
        if !changed {
            for rule in current_rules {
                if !new_rules.contains(rule) {
                    changed = true;
                    break;
                }
            }
        }
        if changed {
            let tags = self.generate(&new_rules);
            let mut wallpapermap: HashMap<Wallpaper, (u32, u8)> = HashMap::new();
            for wallpaper in self.wallpapers.clone() {
                for tag in &wallpaper.tags {
                    if tags.contains_key(tag) {
                        if wallpapermap.contains_key(&wallpaper) {
                            if wallpapermap.get(&wallpaper).unwrap().1 <= tags.get(tag).unwrap().1 {
                                wallpapermap.get_mut(&wallpaper).unwrap().0 +=
                                    tags.get(tag).unwrap().0;
                            } else {
                                wallpapermap
                                    .insert(wallpaper.clone(), tags.get(tag).unwrap().to_owned());
                            }
                        } else {
                            wallpapermap
                                .insert(wallpaper.clone(), tags.get(tag).unwrap().to_owned());
                        }
                    }
                }
            }
            let mut wallpapers = Vec::new();
            let mut distribution = Vec::new();
            for wallpaper in wallpapermap {
                wallpapers.push(wallpaper.0);
                distribution.push(wallpaper.1.0);
            }

            Some((wallpapers, distribution, new_rules))
        } else {
            None
        }
    }

    fn generate(&self, rules: &Vec<Rule>) -> HashMap<Tag, (u32, u8)> {
        let mut min_level: u8 = 0;
        for rule in rules {
            if !rule.transparent && rule.level > min_level {
                min_level = rule.level;
            }
        }
        let ruled = rules.iter().filter(|rule| rule.level >= min_level);
        let mut tags: HashMap<Tag, (u32, u8)> = HashMap::new();
        for rule in ruled {
            let (tags_list, weight, level) = rule.load();
            for tag in tags_list {
                if tags.contains_key(&tag) {
                    if tags.get(&tag).unwrap().1 == level {
                        tags.get_mut(&tag).unwrap().0 += weight;
                    } else {
                        tags.insert(tag, (weight, level));
                    }
                } else {
                    tags.insert(tag, (weight, level));
                }
            }
        }

        tags
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Hash)]
pub struct Tag {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct Wallpaper {
    name: String,
    pub path: PathBuf,
    tags: Vec<Tag>,
    r#type: WallpaperType,
}

impl Wallpaper {
    pub fn new(path: PathBuf, tags: Vec<Tag>) -> Self {
        Self {
            name: path.file_name().unwrap().to_str().unwrap().to_string(),
            path,
            tags,
            r#type: WallpaperType::Picture,
        }
    }

    pub fn new_with_name(name: String, path: PathBuf, tags: Vec<Tag>) -> Self {
        Self {
            name,
            path,
            tags,
            r#type: WallpaperType::Picture,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
enum WallpaperType {
    Picture,
    Animation,
    Video,
}
