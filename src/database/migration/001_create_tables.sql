CREATE TABLE IF NOT EXISTS `user` (
    `id` TEXT PRIMARY KEY NOT NULL,
    `username` TEXT UNIQUE NOT NULL,
    `password` BLOB NOT NULL,
    `login_at` DATETIME,
    `attempts` NUMBER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS `user_i0` ON `user` (`username`);

CREATE TABLE IF NOT EXISTS `account` (
    `id` TEXT PRIMARY KEY NOT NULL,
    `name` TEXT NOT NULL,
    `alias` TEXT NOT NULL,
    `owner` TEXT NOT NULL REFERENCES `user` (`id`),
    `kind` TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS `account_i0` ON `account` (`owner`);

CREATE TABLE IF NOT EXISTS `transaction` (
    `id` TEXT PRIMARY KEY NOT NULL,
    `account` TEXT NOT NULL REFERENCES `account` (`id`),
    `date` DATE NOT NULL,
    `action` TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS `transaction_i0` ON `transaction` (`account`);

CREATE INDEX IF NOT EXISTS `transaction_i1` ON `transaction` (`date`);
