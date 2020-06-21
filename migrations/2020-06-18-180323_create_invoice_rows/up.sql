CREATE TABLE `invoice_rows`
(
    `id`            INT          NOT NULL AUTO_INCREMENT,
    `invoice_id`    INT          NOT NULL,
    `item_name`     VARCHAR(200) NOT NULL,
    `item_price`    FLOAT        NOT NULL,
    `item_count`    TINYINT      NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE (`invoice_id`)
) ENGINE = InnoDB;
