CREATE TABLE IF NOT EXISTS `asset` (
    `id` TEXT PRIMARY KEY NOT NULL,
    `asset_id` TEXT NOT NULL,
    `name` TEXT NOT NULL,
    `extra` TEXT NOT NULL,
    `owner` TEXT REFERENCES `user` (`id`) DEFAULT NULL,
    UNIQUE (`asset_id`, `owner`)
);

CREATE INDEX IF NOT EXISTS `asset_i0` ON `asset` (`owner`);

CREATE TABLE IF NOT EXISTS `asset_price` (
    `asset` TEXT REFERENCES `asset` (`id`) NOT NULL,
    `date` DATE NOT NULL,
    `currency` TEXT NOT NULL,
    `price` NUMBER NOT NULL,
    PRIMARY KEY (`asset`, `date`, `currency`)
);

CREATE INDEX IF NOT EXISTS `asset_price_i0` ON `asset_price` (`asset`);

CREATE TABLE IF NOT EXISTS `asset_dividend` (
    `asset` TEXT REFERENCES `asset` (`id`) NOT NULL,
    `date` DATE NOT NULL,
    `currency` TEXT NOT NULL,
    `price` NUMBER NOT NULL,
    PRIMARY KEY (`asset`, `date`)
);

CREATE INDEX IF NOT EXISTS `asset_dividend_i0` ON `asset_dividend` (`asset`);

CREATE TABLE IF NOT EXISTS `asset_split` (
    `asset` TEXT REFERENCES `asset` (`id`) NOT NULL,
    `date` DATE NOT NULL,
    `ratio` TEXT NOT NULL,
    PRIMARY KEY (`asset`, `date`)
);

CREATE INDEX IF NOT EXISTS `asset_split_i0` ON `asset_split` (`asset`);

CREATE TABLE IF NOT EXISTS `asset_update` (
    `asset` TEXT REFERENCES `asset` (`id`) NOT NULL,
    `query` TEXT NOT NULL,
    `updated_at` DATETIME NOT NULL,
    PRIMARY KEY (`asset`, `query`)
);

CREATE INDEX IF NOT EXISTS `asset_update_i0` ON `asset_update` (`asset`);
