use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SceneType {
    Educational,
    Narrative,
}

#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Setting {
    Classroom,
    Campus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneOutline {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub scene_type: SceneType,
    pub setting: Setting,
    pub summary: String,
    pub learning_objectives: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Storyboard {
    pub scenes: Vec<SceneOutline>,
}
