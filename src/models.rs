//models.rs
use serde::Serialize;


#[derive(Clone, Debug, Serialize)]
pub struct Exercise {
    pub name: String,
    pub sets: u8,
    pub reps: Option<u8>,
    pub each: bool,
    pub seconds: Option<u8>,
    pub weight: Option<f32>
}

#[derive(Clone, Debug, Serialize)]
pub struct Workout {
    pub name: String,
    pub exercises: Vec<Exercise>
}

#[derive(Clone, Debug, Serialize)]
pub struct Conditioning {
    pub name: String,
    pub choices: Vec<Cardio>
}

#[derive(Clone, Debug, Serialize)]
pub struct Cardio {
    pub name: String,
    pub description: String,
    pub time: Option<u8>,
    pub rest: Option<u8>,
    pub sets: u8
}


