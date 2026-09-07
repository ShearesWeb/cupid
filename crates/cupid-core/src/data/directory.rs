//! Decode the complete directory without allocation-specific filtering.
use std::error::Error;

use postgres::GenericClient;

use crate::directory::{Cca, CcaAppointment, CcaPosition, Directory, User};

pub(super) fn load(client: &mut impl GenericClient) -> Result<Directory, Box<dyn Error>> {
    let users = super::users::load(client)?
        .into_iter()
        .map(|u| User {
            id: u.user_id,
            name: u.name,
            email: u.email,
        })
        .collect();
    let ccas = client
        .query(
            "SELECT id, name, kind::text, tier::text, type::text, description, image_url FROM ccas",
            &[],
        )?
        .iter()
        .map(|row| {
            Ok(Cca {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                kind: row.try_get::<_, &str>("kind")?.parse()?,
                tier: row.try_get::<_, &str>("tier")?.parse()?,
                cca_type: row.try_get::<_, &str>("type")?.parse()?,
                description: row.try_get("description")?,
                image_url: row.try_get("image_url")?,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let positions = client.query(
        "SELECT id, cca_id, reporting_position_id, position_type::text, name, description, capacity \
         FROM cca_positions", &[],
    )?.iter().map(|row| Ok(CcaPosition {
        id: row.try_get("id")?, cca_id: row.try_get("cca_id")?,
        reporting_position_id: row.try_get("reporting_position_id")?,
        position_type: row.try_get::<_, &str>("position_type")?.parse()?,
        name: row.try_get("name")?, description: row.try_get("description")?,
        capacity: row.try_get::<_, Option<i32>>("capacity")?.map(usize::try_from).transpose()?,
    })).collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let appointments = client
        .query(
            "SELECT user_id, position_id, commitment_period::text, points, team_status::text, \
                created_at::text FROM cca_appointments",
            &[],
        )?
        .iter()
        .map(|row| {
            Ok(CcaAppointment {
                user_id: row.try_get("user_id")?,
                position_id: row.try_get("position_id")?,
                commitment_period: row.try_get::<_, &str>("commitment_period")?.parse()?,
                points: u32::try_from(row.try_get::<_, i32>("points")?)?,
                team_status: row.try_get::<_, &str>("team_status")?.parse()?,
                created_at: Some(row.try_get("created_at")?),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    Ok(Directory::new(users, ccas, positions, appointments)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::{CommitmentPeriod, PositionKind, TeamStatus};

    #[test]
    #[ignore = "requires CUPID_TEST_DATABASE_URL; creates transaction-local temporary tables only"]
    fn directory_load_against_postgres() {
        let url =
            std::env::var("CUPID_TEST_DATABASE_URL").expect("CUPID_TEST_DATABASE_URL required");
        let mut client = postgres::Client::connect(&url, postgres::NoTls).unwrap();
        let mut tx = client.transaction().unwrap();
        // Temp tables shadow production names only in this connection. Their
        // column types match the schema; enum reads explicitly cast to text.
        tx.batch_execute("
            CREATE TEMP TABLE users (id integer, name text, email text) ON COMMIT DROP;
            CREATE TEMP TABLE ccas (id integer, name text, kind text, tier text, type text,
                description text, image_url text) ON COMMIT DROP;
            CREATE TEMP TABLE cca_positions (id integer, cca_id integer, reporting_position_id integer,
                position_type text, name text, description text, capacity integer) ON COMMIT DROP;
            CREATE TEMP TABLE cca_appointments (user_id integer, position_id integer, commitment_period text,
                points integer, team_status text, created_at timestamptz) ON COMMIT DROP;
            INSERT INTO users VALUES (1, 'Ann', 'ann@x');
            INSERT INTO ccas VALUES
                (1, 'Sports', 'sports', 'tier-1', 'type-b', 'Description', 'https://example.test/image.png'),
                (2, 'Empty', 'adhoc', 'none', 'none', NULL, NULL);
            INSERT INTO cca_positions VALUES
                (10, 1, NULL, 'lead', 'Captain', 'Position description', 1),
                (11, 1, 10, 'vice', 'Vice Captain', NULL, NULL),
                (12, 1, 10, 'blockcomm', 'Block', NULL, 0),
                (13, 1, 10, 'maincomm', 'Main', NULL, 2),
                (14, 1, 10, 'subcomm', 'Sub', NULL, 2),
                (15, 1, 10, 'team-manager', 'Manager', NULL, 1),
                (16, 1, 10, 'member', 'Member', NULL, NULL),
                (17, 1, NULL, 'resident', 'Resident', NULL, NULL);
            INSERT INTO cca_appointments VALUES
                (1, 10, 'semester-2', 17, 'main-team', '2026-08-01T00:00:00Z');
        ").unwrap();
        let directory = load(&mut tx).unwrap();
        assert_eq!(directory.ccas().count(), 2);
        assert_eq!(directory.positions().count(), 8);
        assert_eq!(directory.position(11).unwrap().capacity, None);
        assert_eq!(directory.position(12).unwrap().capacity, Some(0));
        assert_eq!(
            directory.position(11).unwrap().reporting_position_id,
            Some(10)
        );
        assert_eq!(
            directory.position(15).unwrap().position_type,
            PositionKind::TeamManager
        );
        assert_eq!(
            directory.cca(1).unwrap().image_url.as_deref(),
            Some("https://example.test/image.png")
        );
        let appointment = directory.appointment(1, 10).unwrap();
        assert_eq!(appointment.commitment_period, CommitmentPeriod::Semester2);
        assert_eq!(appointment.team_status, TeamStatus::MainTeam);
        assert_eq!(appointment.points, 17);
        assert!(
            appointment
                .created_at
                .as_ref()
                .unwrap()
                .starts_with("2026-08-01")
        );

        tx.batch_execute("UPDATE cca_positions SET position_type = 'unknown' WHERE id = 15")
            .unwrap();
        assert!(
            load(&mut tx).is_err(),
            "unknown types must not silently disappear"
        );
        tx.rollback().unwrap();
    }
}
