CREATE TABLE IF NOT EXISTS sponsors (
	id SERIAL PRIMARY KEY,
	name TEXT NOT NULL,
	active BOOLEAN NOT NULL,
	background_url TEXT NOT NULL,
	private_key TEXT NOT NULL,
	public_key TEXT NOT NULL,
	token_mint TEXT NOT NULL,
	original_tokens INT NOT NULL,
	available_tokens INT NOT NULL,
	reward_tokens INT NOT NULL,
	challenge_time INT NOT NULL,
	system_instruction TEXT NOT NULL,
	greeting_text TEXT NOT NULL,
	challenge_text TEXT NOT NULL,
	start_text TEXT NOT NULL,
	end_text TEXT NOT NULL,
	won_text TEXT NOT NULL,
	lost_text TEXT NOT NULL,
	rating_threshold INT NOT NULL
);

CREATE TABLE IF NOT EXISTS winners (
	id SERIAL PRIMARY KEY,
	key TEXT NOT NULL,
	name TEXT NOT NULL,
	sponsor_id INT NOT NULL REFERENCES sponsors(id)
);

CREATE TABLE IF NOT EXISTS users (
	phone_number TEXT NOT NULL PRIMARY KEY,
	attempts_today INT NOT NULL,
	last_attempt TIMESTAMP WITH TIME ZONE NOT NULL,
	banned BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS attempts (
	id SERIAL PRIMARY KEY,
	pubkey TEXT,
	phone_number TEXT NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
	video_url TEXT, 
	twitter_url TEXT, 
	is_winner BOOLEAN,
	sponsor_question TEXT,
	sponsor_name TEXT,
	sponsor_token_mint TEXT,
	sponsor_total_reward INT,
	sponsor_attempt_reward INT,
	sponsor_background_url TEXT,
	sponsor_challenge_time INT,
	challenge_transcript TEXT, 
	challenge_status TEXT, 
	winner_url TEXT,
	call_sid TEXT
);