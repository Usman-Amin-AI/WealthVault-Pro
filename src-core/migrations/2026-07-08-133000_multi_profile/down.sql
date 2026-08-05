-- Revert app_settings
CREATE TABLE app_settings_old (
    setting_key TEXT PRIMARY KEY NOT NULL,
    setting_value TEXT NOT NULL
);

INSERT INTO app_settings_old (setting_key, setting_value) SELECT setting_key, setting_value FROM app_settings WHERE profile_id = 'default_profile';

DROP TABLE app_settings;
ALTER TABLE app_settings_old RENAME TO app_settings;

-- Remove profile_id columns
ALTER TABLE contribution_limits DROP COLUMN profile_id;
ALTER TABLE goals DROP COLUMN profile_id;
ALTER TABLE accounts DROP COLUMN profile_id;

-- Drop new tables
DROP TABLE profile_shares;
DROP TABLE profiles;
