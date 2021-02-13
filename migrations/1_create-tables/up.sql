CREATE TABLE `entrepreneurs`
(
    `id`                          INT          NOT NULL AUTO_INCREMENT,
    `account_id`                  INT          NOT NULL,
    `code`                        VARCHAR(50)  NOT NULL,
    `name`                        VARCHAR(200) NOT NULL,
    `address`                     VARCHAR(200) NOT NULL,
    `account_number_country_code` VARCHAR(2)   NOT NULL,
    `account_number_prefix`       SMALLINT,
    `vat`                         VARCHAR(50)  NOT NULL,
    `account_number`              BIGINT       NOT NULL,
    `account_bank_code`           SMALLINT     NOT NULL,
    `email`                       VARCHAR(100),
    `phone`                       VARCHAR(20),
    `currency_code`               VARCHAR(10)  NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`code`)
) ENGINE = InnoDB;

CREATE TABLE `accounts`
(
    `id`       INT          NOT NULL AUTO_INCREMENT,
    `username` VARCHAR(100) NOT NULL,
    `password` VARCHAR(64)  NOT NULL,
    `settings` TEXT         NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`username`)
) ENGINE = InnoDB;

CREATE TABLE `contacts`
(
    `id`              INT          NOT NULL AUTO_INCREMENT,
    `entrepreneur_id` INT          NOT NULL,
    `code`            VARCHAR(100),
    `name`            VARCHAR(250) NOT NULL,
    `address`         VARCHAR(250) NOT NULL,
    `vat`             VARCHAR(50)  NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`entrepreneur_id`, `code`, `name`)
) ENGINE = InnoDB;

CREATE TABLE `invoices`
(
    `id`              INT          NOT NULL AUTO_INCREMENT,
    `entrepreneur_id` INT          NOT NULL,
    `contact_id`      INT          NOT NULL,
    `code`            VARCHAR(200) NOT NULL,
    `created`         DATE         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `pay_until`       DATE         NOT NULL,
    `payed`           DATE         NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`code`, `entrepreneur_id`)
) ENGINE = InnoDB;

CREATE TABLE `invoice_rows`
(
    `id`         INT               NOT NULL AUTO_INCREMENT,
    `invoice_id` INT               NOT NULL,
    `item_name`  VARCHAR(400)      NOT NULL,
    `item_price` FLOAT UNSIGNED    NOT NULL,
    `item_count` SMALLINT UNSIGNED NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE = InnoDB;

CREATE TABLE `login_sessions`
(
    `id`         VARCHAR(200) NOT NULL,
    `account_id` INT          NOT NULL,
    PRIMARY KEY (`id`),
    FOREIGN KEY (account_id)
        REFERENCES `accounts` (id)
        ON DELETE CASCADE
) ENGINE = InnoDB;

ALTER TABLE `contacts`
    ADD FOREIGN KEY (`entrepreneur_id`) REFERENCES `entrepreneurs` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `invoices`
    ADD FOREIGN KEY (`entrepreneur_id`) REFERENCES `entrepreneurs` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `entrepreneurs`
    ADD FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `invoices`
    ADD FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE `invoice_rows`
    ADD FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;
