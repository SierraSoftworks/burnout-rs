use crate::models::*;
use super::{AuthToken, APIError};

pub async fn ensure_user_team(state: &GlobalState, token: &AuthToken) -> Result<(), APIError> {
    let uid = u128::from_str_radix(token.oid.replace("-", "").as_str(), 16)
        .or(Err(APIError::new(400, "Bad Request", "The auth token OID you provided could not be parsed. Please check it and try again.")))?;

    match state.store.send(StoreUser {
        principal_id: uid,
        email_hash: u128::from_be_bytes(md5::compute(token.unique_name.to_lowercase().trim().as_bytes()).into()),
        first_name: token.name.splitn(2, " ").nth(0).unwrap_or("").to_string(),
    }).await? {
        Ok(_) => {},
        Err(err) => {
            warn!("Unable to store an entry in the users table for this user: {}", err);
        }
    }

    match state.store.send(GetTeam {
        id: uid,
        principal_id: uid,
    }).await? {
        Ok(_) => {},
        Err(err) => {
            info!("User does not have a default team ({}): {}", uid, err);

            state.store.send(StoreTeam {
                team_id: uid,
                principal_id: uid,
                name: "My Team".into(),
            }).await??;
        }
    }

    state.store.send(StoreTeamAssignment {
        team_id: uid,
        principal_id: uid,
        role: Role::Manager,
    }).await??;

    Ok(())
}