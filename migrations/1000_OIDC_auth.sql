BEGIN; -- Work in a transaction

-- Required for proper OIDC login, and enables redirecting users back to where
-- they were before they were redirected to log in again.
CREATE TABLE LoginProcesses (
	creation_time TIMESTAMPTZ DEFAULT NOW(), -- To clean old processes
	state_id VARCHAR PRIMARY KEY, -- Randomly generated, collisions improbable
	nonce VARCHAR NOT NULL -- Used to validate the OIDC response
);

-- To track our internal state (and registrations) we need to have user entries
CREATE TABLE Users (
	id SERIAL PRIMARY KEY, -- Other tables will link to this
	email VARCHAR NOT NULL UNIQUE -- Logins match on identifier to log in
);

-- OIDC login takes care of authentication and user metadata, but we still need
-- to manage the sessions ourselves.
CREATE TABLE Sessions (
	session_id VARCHAR PRIMARY KEY, -- Randomly generated, collisions improbable
	user_id INTEGER NOT NULL, -- The user this session claims that you are
	valid_until TIMESTAMPTZ DEFAULT NOW() + '6 hours',
	-- Other relevant user metadata we get from OIDC

	FOREIGN KEY (user_id) REFERENCES Users(id)
);

COMMIT; -- Apply the transaction
