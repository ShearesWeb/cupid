//! Domain model for CCA allocation. These types define the complete matcher API;
//! the current binary does not exercise every field, method, or variant yet (e.g.
//! the event-audit queries and position descriptions), so intentionally-unused
//! items are allowed here rather than deleted from the model.
#![allow(dead_code)]

mod allocation;
mod applicant;
mod capacity;
mod ledger;
mod pool;
mod position;

pub use allocation::{Algorithm, MatchResult, RejectReason};
pub use applicant::{Applicant, ApplicantIdx};
pub use capacity::{Appeals, CapacityStore};
pub use ledger::Ledger;
pub use pool::Pool;
pub use position::{CCAIdx, Position, PositionIdx, PositionType};
