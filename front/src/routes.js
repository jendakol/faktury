import Dashboard from "./components/Dashboard";
import InvoiceDetail from "./components/InvoiceDetail";
import ContactDetail from "./components/ContactDetail";
import UserSettings from "./components/UserSettings";
import NotFound from "./components/NotFound";

const routes = [
    {name: "Root", path: '/', redirect: '/dashboard'},
    {name: 'Dashboard', path: '/dashboard', component: Dashboard},
    {name: 'InvoiceDetail', path: '/invoice/:id', component: InvoiceDetail},
    {name: 'ContactDetail', path: '/contact/:id', component: ContactDetail},
    {name: 'UserSettings', path: '/settings', component: UserSettings},
    {path: '*', component: NotFound},
]

export default routes
