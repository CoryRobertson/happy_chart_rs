use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ActivityUIState {
    pub show_activity_screen: bool,
    pub edit_mode: bool,
    pub add_or_remove_mode: bool,
    pub activity_creat_text: String,
    pub added_activity_list: ActivitySelectionList,
}

#[derive(Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct Activity(String);

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Activity {
    pub fn new(activity_name: &str) -> Self {
        Self(activity_name.to_string())
    }

    pub fn get_activity_name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivitySelectionList {
    activity_list: Vec<Activity>,
}

impl ActivitySelectionList {
    pub fn new() -> Self {
        Self {
            activity_list: vec![],
        }
    }

    pub fn get_activity_list(&self) -> &Vec<Activity> {
        &self.activity_list
    }

    pub fn remove_activity(&mut self, activity: &Activity) {
        self.activity_list
            .retain(|act| act.get_activity_name().ne(activity.get_activity_name()));
    }

    pub fn add_new_activity(&mut self, activity: Activity) {
        self.activity_list.push(activity);
        self.activity_list
            .dedup_by(|a1, a2| a1.get_activity_name().eq(a2.get_activity_name()));
        self.activity_list
            .sort_by_key(|act| act.get_activity_name().to_string());
    }
}

impl Default for ActivitySelectionList {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ActivityUIState {
    fn default() -> Self {
        Self {
            show_activity_screen: false,
            edit_mode: false,
            add_or_remove_mode: true,
            activity_creat_text: String::new(),
            added_activity_list: ActivitySelectionList::default(),
        }
    }
}
