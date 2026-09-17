use serde::{Deserialize, Serialize};

use crate::models::PositionType;

// Database spelling is shared by SQL decoding and the serialized directory.
macro_rules! db_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $value)] $variant),+
        }
        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }
        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(format!("Unknown {}: {value}", stringify!($name))),
                }
            }
        }
    };
}

db_enum!(PositionKind {
    Lead => "lead", Vice => "vice", BlockComm => "blockcomm",
    MainComm => "maincomm", SubComm => "subcomm", TeamManager => "team-manager",
    Member => "member", Resident => "resident",
});
db_enum!(CcaKind {
    Sports => "sports", Committee => "committee", Culture => "culture",
    Jcrc => "jcrc", Adhoc => "adhoc", Supplementary => "supplementary",
});
db_enum!(CcaTier { None => "none", Tier1 => "tier-1", Tier2 => "tier-2" });
db_enum!(CcaType { None => "none", TypeA => "type-a", TypeB => "type-b" });
db_enum!(CommitmentPeriod {
    Semester1 => "semester-1", Semester2 => "semester-2",
    FullYear => "full-year", ExShearite => "ex-shearite",
});
db_enum!(TeamStatus {
    None => "none", Shortlisted => "shortlisted", Reserve => "reserve",
    MainTeam => "main-team", Varsity => "varsity",
});

impl PositionKind {
    /// Only these three types enter the existing allocation engine.
    pub fn allocation_type(self) -> Option<PositionType> {
        match self {
            Self::BlockComm => Some(PositionType::BlockComm),
            Self::MainComm => Some(PositionType::MainComm),
            Self::SubComm => Some(PositionType::SubComm),
            _ => None,
        }
    }

    /// Explicit policy, independent of database enum order or allocation eligibility.
    pub fn can_manage_appointments(self) -> bool {
        matches!(
            self,
            Self::Lead
                | Self::Vice
                | Self::BlockComm
                | Self::MainComm
                | Self::SubComm
                | Self::TeamManager
                | Self::Member
        )
    }
}

impl CommitmentPeriod {
    /// Mirrors cca_appointments.semesters, including its ELSE case.
    pub fn semesters(self) -> &'static [u8] {
        match self {
            Self::Semester1 => &[1],
            Self::Semester2 => &[2],
            Self::FullYear | Self::ExShearite => &[1, 2],
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.semesters()
            .iter()
            .any(|s| other.semesters().contains(s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cca {
    pub id: i32,
    pub name: String,
    pub kind: CcaKind,
    pub tier: CcaTier,
    #[serde(rename = "type")]
    pub cca_type: CcaType,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaPosition {
    pub id: i32,
    pub cca_id: i32,
    pub reporting_position_id: Option<i32>,
    pub position_type: PositionKind,
    pub name: String,
    pub description: Option<String>,
    /// NULL capacity is unlimited in appointment management, excluded from allocation.
    pub capacity: Option<usize>,
}

/// Database primary key is (user_id, position_id); there is no appointment ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaAppointment {
    pub user_id: i32,
    pub position_id: i32,
    pub commitment_period: CommitmentPeriod,
    pub points: u32,
    pub team_status: TeamStatus,
    /// Absent for a new in-memory appointment until persistence supplies a timestamp.
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CcaAppointmentStatus {
    Existing,
    Added,
    Modified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaAppointmentView {
    #[serde(flatten)]
    pub appointment: CcaAppointment,
    pub status: CcaAppointmentStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CcaAppointmentChange {
    Add {
        appointment: CcaAppointment,
    },
    Remove {
        user_id: i32,
        position_id: i32,
    },
    ChangePeriod {
        user_id: i32,
        position_id: i32,
        from: CommitmentPeriod,
        to: CommitmentPeriod,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CcaAppointmentChangeSet {
    pub base_sync: String,
    pub changes: Vec<CcaAppointmentChange>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPositionView {
    #[serde(flatten)]
    pub position: CcaPosition,
}

/// Stable, sorted UI data. Allocation rankings and results remain in AllocationSnapshot.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySnapshot {
    pub users: Vec<User>,
    pub ccas: Vec<Cca>,
    pub positions: Vec<DirectoryPositionView>,
    pub appointments: Vec<CcaAppointmentView>,
    pub changes: Vec<CcaAppointmentChange>,
}
