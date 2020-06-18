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

ALTER TABLE `invoice_rows`
    ADD FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT;
