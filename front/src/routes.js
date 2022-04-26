import Dashboard from "./components/DashboardPage";
import Invoices from "./components/InvoicesPage";
import InvoiceDetail from "./components/InvoiceDetail";
import Contacts from "./components/ContactsPage";
import ContactDetail from "./components/ContactDetail";
import UserSettings from "./components/UserSettings";
import NotFound from "./components/NotFound";

const routes = [
    {name: "Root", path: '/', redirect: '/dashboard'},
    {name: 'Dashboard', path: '/dashboard', component: Dashboard},
    {name: 'Invoices', path: '/invoices', component: Invoices},
    {name: 'InvoiceDetail', path: '/invoice/:id', component: InvoiceDetail},
    {name: 'Contacts', path: '/contacts', component: Contacts},
    {name: 'ContactDetail', path: '/contact/:id', component: ContactDetail},
    {name: 'UserSettings', path: '/settings', component: UserSettings},
    {path: '*', component: NotFound},
]

export default routes
