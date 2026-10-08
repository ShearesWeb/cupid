//! Complete CCA directory and validated in-memory appointment management.
//! Persistence must validate against fresh data and enforce database constraints;
//! editing this model alone does not write to the database or publish CSVs.
mod types;

use std::collections::BTreeMap;
use std::fmt;

pub use types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryError {
    UnknownUser(i32),
    UnknownPosition(i32),
    ReadOnlyPosition(i32),
    DuplicateEntity {
        entity: &'static str,
        id: i32,
    },
    DuplicateAppointment {
        user_id: i32,
        position_id: i32,
    },
    MissingAppointment {
        user_id: i32,
        position_id: i32,
    },
    InvalidReference(String),
    CcaConflict {
        user_id: i32,
        cca_id: i32,
        position_id: i32,
    },
    CapacityExceeded {
        position_id: i32,
        semester: u8,
        capacity: usize,
    },
}

impl fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUser(id) => write!(f, "Unknown user: {id}."),
            Self::UnknownPosition(id) => write!(f, "Unknown position: {id}."),
            Self::ReadOnlyPosition(id) => {
                write!(f, "Appointments for position {id} are read-only.")
            }
            Self::DuplicateEntity { entity, id } => write!(f, "Duplicate {entity}: {id}."),
            Self::DuplicateAppointment {
                user_id,
                position_id,
            } => write!(f, "User {user_id} already holds position {position_id}."),
            Self::MissingAppointment {
                user_id,
                position_id,
            } => write!(f, "User {user_id} does not hold position {position_id}."),
            Self::InvalidReference(message) => f.write_str(message),
            Self::CcaConflict {
                user_id,
                cca_id,
                position_id,
            } => write!(
                f,
                "User {user_id} already holds position {position_id} in CCA {cca_id} during this period."
            ),
            Self::CapacityExceeded {
                position_id,
                semester,
                capacity,
            } => write!(
                f,
                "Position {position_id} capacity ({capacity}) would be exceeded in semester {semester}."
            ),
        }
    }
}

impl std::error::Error for DirectoryError {}

/// All persisted entities, including roles that never enter the matcher.
/// Private maps ensure edits cannot bypass validation or leave stale indexes.
#[derive(Debug, Clone)]
pub struct Directory {
    users: BTreeMap<i32, User>,
    ccas: BTreeMap<i32, Cca>,
    positions: BTreeMap<i32, CcaPosition>,
    appointments: BTreeMap<(i32, i32), CcaAppointment>,
}

fn index<T>(
    values: Vec<T>,
    entity: &'static str,
    key: impl Fn(&T) -> i32,
) -> Result<BTreeMap<i32, T>, DirectoryError> {
    let mut map = BTreeMap::new();
    for value in values {
        let id = key(&value);
        if map.insert(id, value).is_some() {
            return Err(DirectoryError::DuplicateEntity { entity, id });
        }
    }
    Ok(map)
}

impl Directory {
    /// Loading preserves existing holdings, including member/resident roles and
    /// historical over-capacity data, so an operator can inspect and correct it.
    /// Duplicate identities and broken references are rejected, never dropped.
    pub fn new(
        users: Vec<User>,
        ccas: Vec<Cca>,
        positions: Vec<CcaPosition>,
        appointments: Vec<CcaAppointment>,
    ) -> Result<Self, DirectoryError> {
        let mut directory = Self {
            users: index(users, "user", |u| u.id)?,
            ccas: index(ccas, "CCA", |c| c.id)?,
            positions: index(positions, "position", |p| p.id)?,
            appointments: BTreeMap::new(),
        };
        for position in directory.positions.values() {
            if !directory.ccas.contains_key(&position.cca_id) {
                return Err(DirectoryError::InvalidReference(format!(
                    "Position {} references unknown CCA {}.",
                    position.id, position.cca_id
                )));
            }
            if let Some(parent_id) = position.reporting_position_id {
                let parent = directory
                    .position(parent_id)
                    .ok_or(DirectoryError::UnknownPosition(parent_id))?;
                if parent.id == position.id {
                    return Err(DirectoryError::InvalidReference(format!(
                        "Invalid reporting position {parent_id} for position {}.",
                        position.id
                    )));
                }
            }
        }
        for appointment in appointments {
            directory.require_pair(appointment.user_id, appointment.position_id)?;
            let key = (appointment.user_id, appointment.position_id);
            if directory.appointments.insert(key, appointment).is_some() {
                return Err(DirectoryError::DuplicateAppointment {
                    user_id: key.0,
                    position_id: key.1,
                });
            }
        }
        Ok(directory)
    }

    pub fn users(&self) -> impl Iterator<Item = &User> {
        self.users.values()
    }
    pub fn ccas(&self) -> impl Iterator<Item = &Cca> {
        self.ccas.values()
    }
    pub fn positions(&self) -> impl Iterator<Item = &CcaPosition> {
        self.positions.values()
    }
    pub fn appointments(&self) -> impl Iterator<Item = &CcaAppointment> {
        self.appointments.values()
    }
    pub fn user(&self, id: i32) -> Option<&User> {
        self.users.get(&id)
    }
    pub fn cca(&self, id: i32) -> Option<&Cca> {
        self.ccas.get(&id)
    }
    pub fn position(&self, id: i32) -> Option<&CcaPosition> {
        self.positions.get(&id)
    }
    pub fn appointment(&self, user_id: i32, position_id: i32) -> Option<&CcaAppointment> {
        self.appointments.get(&(user_id, position_id))
    }
    pub fn held_by(&self, user_id: i32) -> impl Iterator<Item = &CcaAppointment> {
        self.appointments
            .range((user_id, i32::MIN)..=(user_id, i32::MAX))
            .map(|(_, a)| a)
    }
    pub fn holders(&self, position_id: i32) -> impl Iterator<Item = &CcaAppointment> {
        self.appointments
            .values()
            .filter(move |a| a.position_id == position_id)
    }

    fn require_pair(&self, user_id: i32, position_id: i32) -> Result<&CcaPosition, DirectoryError> {
        self.user(user_id)
            .ok_or(DirectoryError::UnknownUser(user_id))?;
        self.position(position_id)
            .ok_or(DirectoryError::UnknownPosition(position_id))
    }

    fn editable_position(
        &self,
        user_id: i32,
        position_id: i32,
    ) -> Result<&CcaPosition, DirectoryError> {
        let position = self.require_pair(user_id, position_id)?;
        if !position.position_type.can_manage_appointments() {
            return Err(DirectoryError::ReadOnlyPosition(position_id));
        }
        Ok(position)
    }

    fn validate_holding(&self, appointment: &CcaAppointment) -> Result<(), DirectoryError> {
        let position = self.editable_position(appointment.user_id, appointment.position_id)?;
        let others = || {
            self.appointments.values().filter(|a| {
                (a.user_id, a.position_id) != (appointment.user_id, appointment.position_id)
            })
        };
        for other in others() {
            let other_position = &self.positions[&other.position_id];
            if other.user_id == appointment.user_id
                && other_position.cca_id == position.cca_id
                && other_position.position_type != PositionKind::Resident
                && other
                    .commitment_period
                    .overlaps(appointment.commitment_period)
            {
                return Err(DirectoryError::CcaConflict {
                    user_id: appointment.user_id,
                    cca_id: position.cca_id,
                    position_id: other.position_id,
                });
            }
        }
        if let Some(capacity) = position.capacity {
            for &semester in appointment.commitment_period.semesters() {
                let occupied = others()
                    .filter(|a| {
                        a.position_id == position.id
                            && a.commitment_period.semesters().contains(&semester)
                    })
                    .count();
                if occupied >= capacity {
                    return Err(DirectoryError::CapacityExceeded {
                        position_id: position.id,
                        semester,
                        capacity,
                    });
                }
            }
        }
        Ok(())
    }

    /// A brand-new holding: points, team status and creation time are the
    /// database's to supply, so they start empty.
    pub fn add_appointment(
        &mut self,
        user_id: i32,
        position_id: i32,
        commitment_period: CommitmentPeriod,
    ) -> Result<(), DirectoryError> {
        self.insert_appointment(CcaAppointment {
            user_id,
            position_id,
            commitment_period,
            points: 0,
            team_status: TeamStatus::None,
            created_at: None,
        })
    }

    /// Reinstates a holding the database still has, carrying its own points,
    /// team status and creation time back in. Removing a holder and putting
    /// them back undoes the edit; routing that through `add_appointment`
    /// would silently reset all three to a new holding's blanks.
    pub fn restore_appointment(
        &mut self,
        appointment: CcaAppointment,
    ) -> Result<(), DirectoryError> {
        self.insert_appointment(appointment)
    }

    fn insert_appointment(&mut self, appointment: CcaAppointment) -> Result<(), DirectoryError> {
        let (user_id, position_id) = (appointment.user_id, appointment.position_id);
        self.editable_position(user_id, position_id)?;
        if self.appointment(user_id, position_id).is_some() {
            return Err(DirectoryError::DuplicateAppointment {
                user_id,
                position_id,
            });
        }
        self.validate_holding(&appointment)?;
        self.appointments
            .insert((user_id, position_id), appointment);
        Ok(())
    }

    /// Updates only the commitment period; preserves points, team status and creation time.
    pub fn update_appointment_period(
        &mut self,
        user_id: i32,
        position_id: i32,
        period: CommitmentPeriod,
    ) -> Result<(), DirectoryError> {
        self.editable_position(user_id, position_id)?;
        let mut appointment = self
            .appointment(user_id, position_id)
            .ok_or(DirectoryError::MissingAppointment {
                user_id,
                position_id,
            })?
            .clone();
        appointment.commitment_period = period;
        self.validate_holding(&appointment)?;
        self.appointments
            .insert((user_id, position_id), appointment);
        Ok(())
    }

    /// Missing pairs are a no-op, but never bypass the role policy.
    pub fn remove_appointment(
        &mut self,
        user_id: i32,
        position_id: i32,
    ) -> Result<bool, DirectoryError> {
        self.editable_position(user_id, position_id)?;
        Ok(self.appointments.remove(&(user_id, position_id)).is_some())
    }

    pub fn snapshot(&self) -> DirectorySnapshot {
        self.snapshot_with_changes(&CcaAppointmentChangeSet::default())
    }

    pub fn snapshot_with_changes(&self, changes: &CcaAppointmentChangeSet) -> DirectorySnapshot {
        let changed: std::collections::HashMap<(i32, i32), CcaAppointmentStatus> = changes
            .changes
            .iter()
            .filter_map(|change| match change {
                CcaAppointmentChange::Add { appointment } => Some((
                    (appointment.user_id, appointment.position_id),
                    CcaAppointmentStatus::Added,
                )),
                CcaAppointmentChange::ChangePeriod {
                    user_id,
                    position_id,
                    ..
                } => Some(((*user_id, *position_id), CcaAppointmentStatus::Modified)),
                CcaAppointmentChange::Remove { .. } => None,
            })
            .collect();
        DirectorySnapshot {
            users: self.users.values().cloned().collect(),
            ccas: self.ccas.values().cloned().collect(),
            positions: self
                .positions
                .values()
                .map(|p| DirectoryPositionView {
                    position: p.clone(),
                })
                .collect(),
            appointments: self
                .appointments
                .values()
                .cloned()
                .map(|appointment| {
                    let status = changed
                        .get(&(appointment.user_id, appointment.position_id))
                        .copied()
                        .unwrap_or(CcaAppointmentStatus::Existing);
                    CcaAppointmentView {
                        appointment,
                        status,
                    }
                })
                .collect(),
            changes: changes.changes.clone(),
        }
    }
}
