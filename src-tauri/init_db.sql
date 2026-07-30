CREATE DATABASE IF NOT EXISTS `ai-listen-rs` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE `ai-listen-rs`;
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(64) PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    email VARCHAR(128) DEFAULT NULL,
    phone VARCHAR(20) DEFAULT NULL,
    password_hash VARCHAR(255) NOT NULL DEFAULT '',
    created_at VARCHAR(32) NOT NULL,
    is_vip TINYINT(1) NOT NULL DEFAULT 0,
    provider VARCHAR(32) DEFAULT NULL,
    provider_user_id VARCHAR(128) DEFAULT NULL,
    avatar VARCHAR(512) DEFAULT NULL,
    UNIQUE KEY uk_username (username),
    UNIQUE KEY uk_email (email),
    UNIQUE KEY uk_phone (phone),
    KEY idx_provider (provider, provider_user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
SHOW TABLES;
