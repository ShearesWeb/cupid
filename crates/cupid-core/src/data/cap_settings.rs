use std::error::Error;
use std::path::Path;

use crate::directory::CapSettings;

/// Read the cap settings file (appeals and counting window). Like the
/// preallocation store, these are operator decisions local to this machine.
/// A missing file means defaults: academic-year window, no appeals.
pub fn read_file(path: &Path) -> Result<CapSettings, Box<dyn Error>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(CapSettings::default()),
        Err(e) => Err(e.into()),
    }
}

/// Write the cap settings file, creating parent directories as needed.
pub fn write_file(path: &Path, settings: &CapSettings) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(std::fs::write(path, serde_json::to_vec_pretty(settings)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::CapWindow;

    #[test]
    fn missing_file_is_default_and_round_trips() {
        let dir = std::env::temp_dir().join(format!("cupid-caps-{}", std::process::id()));
        let path = dir.join("appeals.json");
        assert_eq!(read_file(&path).unwrap(), CapSettings::default());
        let settings = CapSettings {
            window: CapWindow::Semester,
            committee_appeals: [7, 3].into(),
            combined_appeals: [5].into(),
            rules: crate::directory::CapRules {
                academic_year: "AY27/28".into(),
                total_max: 5,
                ..Default::default()
            },
        };
        write_file(&path, &settings).unwrap();
        assert_eq!(read_file(&path).unwrap(), settings);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn files_written_before_combined_appeals_still_load() {
        let settings: CapSettings =
            serde_json::from_str(r#"{"window":"semester","committeeAppeals":[2]}"#).unwrap();
        assert_eq!(settings.committee_appeals, [2].into());
        assert!(settings.combined_appeals.is_empty());
    }
}
