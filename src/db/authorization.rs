use rand::RngCore;
use serde::{Deserialize, Serialize};

use super::Executor;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub username: String,
}

impl User {
	pub async fn create(username: String, con: impl Executor<'_>) -> sqlx::Result<Self> {
		sqlx::query!("INSERT INTO users (username) VALUES ($1)", username)
			.execute(con)
			.await?;

		Ok(Self { username })
	}

	pub async fn load(username: &str, con: impl Executor<'_>) -> sqlx::Result<Option<Self>> {
		sqlx::query_as!(Self, "SELECT * FROM users WHERE username = $1", username)
			.fetch_optional(con)
			.await
	}
}

#[derive(Debug)]
pub struct Authentication {
	pub username: String,
	hash: Vec<u8>,
	salt: Vec<u8>,
	pub version: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("Error interacting with database: {0}")]
	Sqlx(#[from] sqlx::Error),
	#[error("Error hashing/verifying password with argon2: {0}")]
	Argon2(#[from] argon2::Error),
}

pub type Result<T> = ::core::result::Result<T, Error>;

impl Authentication {
	pub async fn load(username: &str, con: impl Executor<'_>) -> Result<Option<Self>> {
		sqlx::query_as!(
			Self,
			r#"SELECT
				username,
				hash,
				salt,
				version as "version: u32"
			FROM authentication WHERE username = $1"#,
			username
		)
		.fetch_optional(con)
		.await
		.map_err(Error::Sqlx)
	}

	pub fn verify(&self, password: &str, config: &argon2::Config<'_>) -> Result<bool> {
		argon2::verify_raw(password.as_bytes(), &self.salt, &self.hash, config)
			.map_err(Error::Argon2)
	}

	pub async fn create(
		user: &User,
		password: &str,
		config: &argon2::Config<'_>,
		con: impl Executor<'_>,
	) -> Result<Self> {
		let mut salt = Vec::with_capacity(16);
		salt.resize(16, 0);

		rand::thread_rng().fill_bytes(&mut salt);

		let hash = argon2::hash_raw(password.as_bytes(), &salt, config)?;

		let this = Self {
			username: user.username.clone(),
			hash,
			salt,
			version: 0,
		};

		this.insert(con).await?;

		Ok(this)
	}

	async fn insert(&self, con: impl Executor<'_>) -> Result<()> {
		sqlx::query!(
			"INSERT INTO authentication (username, hash, salt, version) VALUES ($1, $2, $3, $4)",
			self.username,
			self.hash,
			self.salt,
			self.version
		)
		.execute(con)
		.await?;

		Ok(())
	}

	pub async fn update(&self, con: impl Executor<'_>) -> Result<()> {
		sqlx::query!(
			"UPDATE authentication SET (
				username,
				hash,
				salt,
				version
			) = ( $1, $2, $3, $4 )
			WHERE username = $1",
			self.username,
			self.hash,
			self.salt,
			self.version
		)
		.execute(con)
		.await?;

		Ok(())
	}

	pub async fn increment(&mut self, con: impl Executor<'_>) -> Result<()> {
		self.version += 1;

		self.update(con).await
	}

	pub async fn change_password(
		&mut self,
		password: &str,
		config: &argon2::Config<'_>,
		con: impl Executor<'_>,
	) -> Result<()> {
		self.hash = argon2::hash_raw(password.as_bytes(), &self.salt, config)?;

		self.increment(con).await
	}
}
