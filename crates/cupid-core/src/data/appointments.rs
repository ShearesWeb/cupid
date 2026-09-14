/// Allocation projection of an existing appointment and its holder's identity.
/// `commitment_period` is intentionally omitted from this projection (cupid
/// treats every appointment as full-year); the directory preserves it. The
/// `cca_id` and `position_type` columns feed the one-per-CCA rule even for
/// positions outside cupid's market.
#[derive(Debug)]
pub struct AppointmentRecord {
    pub user_id: i32,
    pub user_name: String,
    pub user_email: String,
    pub position_id: i32,
    pub cca_id: i32,
    pub position_type: String,
}
