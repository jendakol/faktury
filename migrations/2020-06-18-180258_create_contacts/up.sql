CREATE TABLE `contacts`
(
    `id`              INT          NOT NULL AUTO_INCREMENT,
    `code`            VARCHAR(100),
    `entrepreneur_id` INT          NOT NULL,
    `name`            VARCHAR(250) NOT NULL,
    `address`         VARCHAR(250) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`code`, `name`)
) ENGINE = InnoDB;
