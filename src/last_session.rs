use serde::{Deserialize, Serialize};

#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct LastSession {
    pub graph_xscale: f32,
    pub graph_yscale: f32,
    pub xoffset: i32,
    pub displaying_day_lines: bool
}

impl Default for  LastSession {
    fn default() -> Self {
        LastSession{
            graph_xscale: 1.0,
            graph_yscale: 1.0,
            xoffset: 0,
            displaying_day_lines: false,
        }
    }
}

