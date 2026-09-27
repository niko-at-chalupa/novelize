use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SceneOutline {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub scene_type: String,
    pub setting: String,
    pub summary: String,
    pub learning_objectives: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Storyboard {
    pub scenes: Vec<SceneOutline>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_open_ended_scene_type_and_setting_values() {
        let json = r#"{
            "scenes": [{
                "id": "scene_1",
                "title": "A new place",
                "type": "historical",
                "setting": "images/bg/class.png",
                "summary": "I enter the classroom.",
                "learning_objectives": []
            }]
        }"#;

        let storyboard: Storyboard = serde_json::from_str(json).unwrap();
        assert_eq!(storyboard.scenes[0].scene_type, "historical");
        assert_eq!(storyboard.scenes[0].setting, "images/bg/class.png");
    }
}
