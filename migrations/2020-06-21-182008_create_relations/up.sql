ALTER TABLE `contacts`
    ADD FOREIGN KEY (`entrepreneur_id`) REFERENCES `entrepreneurs` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE `invoices`
    ADD FOREIGN KEY (`entrepreneur_id`) REFERENCES `entrepreneurs` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE `invoices`
    ADD FOREIGN KEY (`contact_id`) REFERENCES `contacts` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT;

ALTER TABLE `invoice_rows`
    ADD FOREIGN KEY (`invoice_id`) REFERENCES `invoices` (`id`) ON DELETE CASCADE ON UPDATE RESTRICT;
