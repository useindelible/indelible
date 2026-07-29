// sqlx::migrate! embeds the migrations directory at compile time, but cargo does not
// invalidate this crate when a new .sql file appears, leaving binaries with a stale
// embedded migrator that rejects the database as ahead of itself.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
