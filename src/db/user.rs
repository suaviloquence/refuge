use serde::{Deserialize, Serialize};

use super::Executor;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub username: String,
}

#[derive(Debug)]
pub struct Authentication {
	pub username: String,
	pub hash: Box<[u8]>,
	pub salt: Box<[u8]>,
	pub version: u32,
}

impl Authentication {
	pub async fn load(username: &str, con: impl Executor<'_>) -> sqlx::Result<Option<Self>> {
		sqlx::query_as!(
			Self,
			r#"SELECT
				username,
				hash as "hash: Box<[u8]>",
				salt as "salt: Box<[u8]>",
				version as "version: u32"
			FROM authentication WHERE username = $1"#,
			username
		)
		.fetch_optional(con)
		.await
	}
}
