use serde::{Deserialize, Serialize};

use super::Executor;

#[inline(always)]
const fn idx_default() -> i64 {
	-1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
	pub id: i64,
	#[serde(skip, default)]
	pub username: String,
	pub text: String,
	#[serde(skip, default = "idx_default")]
	pub idx: i64,
	pub completed: bool,
	pub cleared: bool,
}

impl Todo {
	pub async fn get_by_id(id: i64, con: impl Executor<'_>) -> sqlx::Result<Option<Self>> {
		sqlx::query_as!(Self, "SELECT * FROM todos WHERE id = $1", id)
			.fetch_optional(con)
			.await
	}

	pub async fn get_by_username(
		username: &str,
		con: impl Executor<'_>,
	) -> sqlx::Result<Vec<Self>> {
		sqlx::query_as!(
			Self,
			"SELECT * FROM todos WHERE username = $1 AND NOT cleared ORDER BY idx, id DESC",
			username
		)
		.fetch_all(con)
		.await
	}

	pub async fn create(
		username: String,
		text: String,
		idx: i64,
		completed: bool,
		cleared: bool,
		con: impl Executor<'_>,
	) -> sqlx::Result<Self> {
		let id = sqlx::query!(
			"INSERT INTO todos (
				username,
				text,
				idx,
				completed,
				cleared
			) VALUES ($1, $2, $3, $4, $5) RETURNING id",
			username,
			text,
			idx,
			completed,
			cleared,
		)
		.fetch_one(con)
		.await?
		.id;

		Ok(Self {
			id,
			username,
			text,
			idx,
			completed,
			cleared,
		})
	}

	pub async fn update(&self, con: impl Executor<'_>) -> sqlx::Result<()> {
		sqlx::query!(
			"UPDATE todos SET (
				id,
				username,
				text,
				idx,
				completed
			) = ($1, $2, $3, $4, $5)
			WHERE id = $1",
			self.id,
			self.username,
			self.text,
			self.idx,
			self.completed,
		)
		.execute(con)
		.await?;

		Ok(())
	}
}
