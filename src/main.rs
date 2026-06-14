mod algorithm;
mod data;
mod models;

use algorithm::run;
use models::{Appeals, Applicant, Position};

fn main() {
    let applicants: Vec<Applicant> = Vec::new();
    let positions: Vec<Position> = Vec::new();
    let appeals = Appeals::new();
    let _result = run(&applicants, &positions, &appeals);
}
