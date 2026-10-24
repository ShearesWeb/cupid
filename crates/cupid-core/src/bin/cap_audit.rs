//! Audit every resident in the database against the CCA caps, the same check
//! the desktop app's CCA caps page runs. Read-only.
//!
//!   DATABASE_URL=postgres://… cargo run -p cupid --bin cap_audit
//!
//! or with the details the app's connect form takes: SUPABASE_PROJECT_REF,
//! SUPABASE_PASSWORD and, optionally, SUPABASE_REGION.
//!
//! CAP_SETTINGS_FILE points at the desktop app's appeals.json (appeals,
//! counting window and this year's rules); absent means the AY26/27 white
//! paper rules with no appeals. On Windows the app keeps it at
//! %APPDATA%\app.shearesweb.cupid\appeals.json.
use std::path::Path;

use cupid::data::cap_settings;
use cupid::data::conn::ConnSpec;
use cupid::directory::{CapSettings, CapWindow, CommitteeCombo, Directory};

fn combos(list: &[CommitteeCombo]) -> String {
    list.iter()
        .map(|c| match (c.main, c.sub) {
            (0, s) => format!("{s}S"),
            (m, 0) => format!("{m}M"),
            (m, s) => format!("{m}M{s}S"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn cca_name(directory: &Directory, id: i32) -> String {
    directory.cca(id).map_or_else(|| format!("CCA {id}"), |c| c.name.clone())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let spec = match (env("DATABASE_URL"), env("SUPABASE_PROJECT_REF"), env("SUPABASE_PASSWORD")) {
        (Some(url), _, _) => ConnSpec::Url(url),
        (None, Some(project_ref), Some(password)) => {
            ConnSpec::supabase(&project_ref, &password, env("SUPABASE_REGION").as_deref())
        }
        _ => return Err("Set DATABASE_URL, or SUPABASE_PROJECT_REF and SUPABASE_PASSWORD (and SUPABASE_REGION if needed).".into()),
    };
    let settings = match std::env::var("CAP_SETTINGS_FILE") {
        Ok(path) => cap_settings::read_file(Path::new(&path))?,
        Err(_) => CapSettings::default(),
    };
    settings.rules.validate()?;
    let directory = cupid::data::db::load_all(&spec)?.directory;
    let rules = &settings.rules;

    println!("CCA cap audit, {} rules", if rules.academic_year.is_empty() { "unnamed" } else { &rules.academic_year });
    println!(
        "  committee: {} (with appeal: {})",
        combos(&rules.committee_combos),
        combos(&rules.committee_combos_appealed)
    );
    println!(
        "  Sports/Culture: {} (with CCA cap appeal: {})",
        rules.sports_culture_max, rules.sports_culture_max_appealed
    );
    println!(
        "  total CCAs: {} (with CCA cap appeal: {}, with both appeals: {})",
        rules.total_max, rules.total_max_appealed, rules.total_max_both_appeals
    );
    println!(
        "  counted per {}",
        match settings.window {
            CapWindow::AcademicYear => "academic year",
            CapWindow::Semester => "semester",
        }
    );
    println!();

    let mut checked = 0;
    let (mut committee, mut sports_culture, mut total, mut duplicate) = (0, 0, 0, 0);
    let mut flagged = Vec::new();
    for user in directory.users() {
        let Some(entry) = directory.committee_cap_for(user.id, &settings) else { continue };
        checked += 1;
        let over_sc = entry.counts.iter().any(|c| c.sports_culture > c.sports_culture_limit);
        let over_total = entry.counts.iter().any(|c| c.main + c.sub + c.sports_culture > c.total_limit);
        committee += usize::from(entry.over_cap);
        sports_culture += usize::from(over_sc);
        total += usize::from(over_total);
        duplicate += usize::from(!entry.duplicate_ccas.is_empty());
        if entry.over_cap || entry.over_combined_cap || !entry.duplicate_ccas.is_empty() {
            let mut why = Vec::new();
            if entry.over_cap {
                why.push(if entry.appealed { "over committee appeal cap" } else { "over committee cap" });
            }
            if over_sc {
                why.push("over Sports/Culture cap");
            }
            if over_total {
                why.push("over total cap");
            }
            if !entry.duplicate_ccas.is_empty() {
                why.push("same committee twice");
            }
            flagged.push((user.name.clone(), user.email.clone(), entry, why.join(", ")));
        }
    }

    println!("Residents with a counted CCA: {checked}");
    println!("Within every cap:             {}", checked - flagged.len());
    println!("Over committee cap:           {committee}");
    println!("Over Sports/Culture cap:      {sports_culture}");
    println!("Over total cap:               {total}");
    println!("Same committee twice:         {duplicate}");
    println!(
        "Approved appeals on file:     {} committee, {} CCA cap",
        settings.committee_appeals.len(),
        settings.combined_appeals.len()
    );
    let report = directory.committee_caps(&settings);
    if !report.unclassified_ccas.is_empty() {
        let names: Vec<_> = report.unclassified_ccas.iter().map(|&id| cca_name(&directory, id)).collect();
        println!("Committees with no Type A/B:  {} ({})", names.len(), names.join(", "));
    }

    if flagged.is_empty() {
        println!("\nEveryone is within the caps.");
        return Ok(());
    }
    flagged.sort_by(|a, b| a.0.cmp(&b.0));
    println!("\nResidents to review ({}):", flagged.len());
    for (name, email, entry, why) in flagged {
        let counts: Vec<_> = entry
            .counts
            .iter()
            .map(|c| {
                let prefix = c.semester.map_or_else(String::new, |s| format!("Sem {s}: "));
                format!(
                    "{prefix}{}M {}S + {} Sports/Culture = {}",
                    c.main,
                    c.sub,
                    c.sports_culture,
                    c.main + c.sub + c.sports_culture
                )
            })
            .collect();
        println!("- {name} <{email}>: {why}");
        println!("    {}", counts.join("; "));
        let seats: Vec<_> = entry
            .seats
            .iter()
            .map(|s| {
                let position = directory.position(s.position_id).map_or("?", |p| p.name.as_str());
                format!("{} {}", cca_name(&directory, s.cca_id), position)
            })
            .chain(entry.sports_culture_ccas.iter().map(|&id| cca_name(&directory, id)))
            .collect();
        println!("    {}", seats.join(" | "));
    }
    Ok(())
}
