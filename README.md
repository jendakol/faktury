# Faktury

_Czech term for 'invoices'_

A very simple web app (web UI + backend) for management of small entrepreneurs invoices.

The app is in very early development stage, don't expect too much from it (yet).

## Feature set
[ ] Register yourself as an entrepreneur (fill in you details)
[ ] Create set of your business contacts
[ ] Create an invoice for specified contact, add _rows_ with a name, price, count
[ ] Generate a PDF file for the invoice
[ ] View stats for your business
[ ] Customizable PDF layout
[ ] Period summary for accounting
[ ] Internationalized

## Main/interesting used technologies

### Backend
* [Rust](https://www.rust-lang.org/)
* [Actix.rs](https://actix.rs/) (web framework)
* [Diesel](https://diesel.rs/) (DB connection, ORM)
* [Serde](https://serde.rs/) (JSON [de]?serialization)
* [Printpdf](https://docs.rs/printpdf/) (PDF generation)

The MySQL ([MariaDB](https://mariadb.org/)) was used as a DB server.

### Frontend
* [Vue.js](https://vuejs.org/) (basic framework)
* [Vuetify](https://vuetifyjs.com/) (components)
* [Vue Router](https://router.vuejs.org/) (pages routing)
* [Vuex](vuex.vuejs.org/) (state handling)
* [Vue Snotify](https://github.com/artemsky/vue-snotify) (notifications)
* [Axios](https://github.com/axios/axios) (AJAX loading)
