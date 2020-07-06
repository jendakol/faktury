CREATE TABLE `invoices`
(
    `id`              INT          NOT NULL AUTO_INCREMENT,
    `code`            VARCHAR(200) NOT NULL,
    `entrepreneur_id` INT          NOT NULL,
    `contact_id`      INT          NOT NULL,
    `created`         DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `pay_until`       DATE         NOT NULL,
    `payed`           DATE         NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`code`)
) ENGINE = InnoDB;
