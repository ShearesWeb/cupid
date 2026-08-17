//! Domain model for CCA allocation. These types define the complete matcher API;
//! the current binary does not exercise every field, method, or variant yet (e.g.
//! the event-audit queries and position descriptions), so intentionally-unused
//! items are allowed here rather than deleted from the model.
#![allow(dead_code)]

mod allocation;
mod applicant;
mod appointment;
mod capacity;
mod cca;
mod ledger;
mod pool;
mod position;
mod preallocation;

pub use allocation::{Algorithm, Allocation, Event, EventKind, MatchResult, RejectReason};
pub use applicant::{Applicant, ApplicantIdx};
pub use appointment::{Appointment, Appointments};
pub use capacity::{CapacityStore, HeldCounts};
pub use cca::{Cca, CcaIdx};
pub use ledger::Ledger;
pub use pool::{Pool, Roster};
pub use position::{Position, PositionIdx, PositionType};
pub use preallocation::Preallocations;
