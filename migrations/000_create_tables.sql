CREATE TABLE users (
	username TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE authentication (
	username TEXT NOT NULL PRIMARY KEY REFERENCES users(username) ON DELETE CASCADE,
	hash BLOB NOT NULL,
	salt BLOB NOT NULL,
	version INTEGER NOT NULL
);

CREATE TABLE todos (
	id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
	username TEXT NOT NULL REFERENCES users(username) ON DELETE CASCADE,
	text TEXT NOT NULL,
	idx INTEGER NOT NULL
);
