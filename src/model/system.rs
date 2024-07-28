use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct SystemSetting {
    pub name: String,
    pub value: String,
    pub description: String,
}
