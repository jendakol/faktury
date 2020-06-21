CREATE TABLE `entrepreneurs`
(
    `id`      INT          NOT NULL AUTO_INCREMENT,
    `code`    VARCHAR(200) NOT NULL,
    `name`    VARCHAR(200) NOT NULL,
    `address` VARCHAR(200) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`code`)
) ENGINE = InnoDB;
