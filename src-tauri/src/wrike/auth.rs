use keyring::Entry;

const SERVICE: &str = "com.valveandmeter.tally";
const USERNAME: &str = "wrike-token";

/// The token is the only secret here, so it's the only thing that goes in
/// the OS keychain. Everything else (host, display name, contact id) is
/// non-secret and lives in the regular SQLite cache (see db::WrikeAccount).
pub fn save_token(token: &str) -> Result<(), String> {
    Entry::new(SERVICE, USERNAME)
        .and_then(|entry| entry.set_password(token))
        .map_err(|e| format!("Could not save your Wrike token to the keychain: {e}"))
}

pub fn load_token() -> Option<String> {
    Entry::new(SERVICE, USERNAME)
        .ok()
        .and_then(|entry| entry.get_password().ok())
}

pub fn clear_token() {
    if let Ok(entry) = Entry::new(SERVICE, USERNAME) {
        let _ = entry.delete_credential();
    }
}
