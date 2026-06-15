use std::error::Error;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

/// Templates carrying this suffix are skipped by `load_dir`.
const EXAMPLE_SUFFIX: &str = ".example.csv";

/// Parse CSV records of generic type `T` from any reader.
pub fn parse<T: DeserializeOwned, R: Read>(reader: R) -> Result<Vec<T>, Box<dyn Error>> {
    let mut rdr = ::csv::ReaderBuilder::new()
        .trim(::csv::Trim::All)
        .from_reader(reader);
    rdr.deserialize::<T>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Parse every `*.csv` (excluding `*.example.csv`) in `dir` as records of generic type `T`.
pub fn load_dir<T: DeserializeOwned>(dir: &Path) -> Result<Vec<T>, Box<dyn Error>> {
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .map(|entry| entry.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|p| is_data_csv(p))
            .collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    paths.sort();

    let mut records = Vec::new();
    for path in &paths {
        records.extend(load_file::<T>(path)?);
    }
    Ok(records)
}

/// Read + parse one CSV file as records of type `T`.
fn load_file<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, Box<dyn Error>> {
    let file = std::fs::File::open(path)?;
    parse(file)
}

/// True for a `.csv` file that is not a `*.example.csv` template.
fn is_data_csv(path: &Path) -> bool {
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name.ends_with(".csv") && !name.ends_with(EXAMPLE_SUFFIX),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[derive(Debug, serde::Deserialize)]
    struct Row {
        name: String,
    }

    #[test]
    fn parse_reads_typed_records() {
        let rows: Vec<Row> = parse(Cursor::new("name\nCara\nBen\n")).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Cara");
    }

    #[test]
    fn parse_trims_headers_and_fields() {
        // Header `  name  ` must trim to match the struct field; the value too.
        let rows: Vec<Row> = parse(Cursor::new("  name  \n  Cara  \n")).unwrap();
        assert_eq!(rows[0].name, "Cara");
    }

    #[test]
    fn is_data_csv_skips_examples_and_non_csv() {
        assert!(is_data_csv(Path::new("data/appeals/round1.csv")));
        assert!(!is_data_csv(Path::new("data/appeals/appeals.example.csv")));
        assert!(!is_data_csv(Path::new("data/appeals/notes.txt")));
    }

    #[test]
    fn missing_dir_is_zero_files() {
        let rows: Vec<Row> = load_dir(Path::new("does/not/exist")).unwrap();
        assert!(rows.is_empty());
    }

    #[test]
    fn load_dir_concatenates_sorted_skipping_templates() {
        // Unique temp dir per process — no extra dev-dependency needed.
        let dir = std::env::temp_dir().join(format!("cupid_csv_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join("b.csv"), "name\nBen\n").unwrap();
        std::fs::write(dir.join("a.csv"), "name\nAnn\n").unwrap();
        std::fs::write(dir.join("data.example.csv"), "name\nTEMPLATE\n").unwrap();
        std::fs::write(dir.join("notes.txt"), "ignore me").unwrap();

        let rows: Vec<Row> = load_dir(&dir).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        // a.csv before b.csv (sorted); example + txt excluded.
        let names: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["Ann", "Ben"]);
    }
}
