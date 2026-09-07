/// One row of `cca_positions` joined to its CCA. The market is this table, not
/// the chair shortlists: residents rank positions before any chair shortlists
/// a candidate, so a shortlist-derived universe would drop those picks.
#[derive(Debug)]
pub struct PositionRecord {
    pub position_id: i32,
    pub position_name: String,
    pub position_type: String,
    pub capacity: Option<i32>,
    pub cca_id: i32,
    pub cca_name: String,
}
