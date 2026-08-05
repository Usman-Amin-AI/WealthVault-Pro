-- Create profiles table
CREATE TABLE profiles (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    password_hash TEXT,
    encryption_salt TEXT,
    encryption_key TEXT,
    sharing_rule TEXT NOT NULL DEFAULT 'private',
    family_mode_enabled BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create profile_shares table
CREATE TABLE profile_shares (
    id TEXT PRIMARY KEY NOT NULL,
    owner_profile_id TEXT NOT NULL,
    shared_profile_id TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    permissions TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(owner_profile_id) REFERENCES profiles(id),
    FOREIGN KEY(shared_profile_id) REFERENCES profiles(id)
);

-- Insert a default profile for existing data
INSERT INTO profiles (id, name, created_at) VALUES ('default_profile', 'Default Profile', CURRENT_TIMESTAMP);

-- Add profile_id to accounts
ALTER TABLE accounts ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'default_profile' REFERENCES profiles(id);

-- Add profile_id to goals
ALTER TABLE goals ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'default_profile' REFERENCES profiles(id);

-- Add profile_id to contribution_limits
ALTER TABLE contribution_limits ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'default_profile' REFERENCES profiles(id);

-- Migrate app_settings
CREATE TABLE app_settings_new (
    profile_id TEXT NOT NULL DEFAULT 'default_profile' REFERENCES profiles(id),
    setting_key TEXT NOT NULL,
    setting_value TEXT NOT NULL,
    PRIMARY KEY (profile_id, setting_key)
);

INSERT INTO app_settings_new (setting_key, setting_value) SELECT setting_key, setting_value FROM app_settings;

DROP TABLE app_settings;
ALTER TABLE app_settings_new RENAME TO app_settings;
