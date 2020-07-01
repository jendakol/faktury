import Dashboard from "./components/Dashboard";
import InvoiceDetail from "./components/InvoiceDetail";
import UserSettings from "./components/UserSettings";
import NotFound from "./components/NotFound";

const routes = [
    {name: "Root", path: '/', redirect: '/dashboard'},
    {name: 'Dashboard', path: '/dashboard', component: Dashboard},
    {name: 'InvoiceDetail', path: '/invoice/:id', component: InvoiceDetail},
    {name: 'UserSettings', path: '/settings', component: UserSettings},
    {path: '*', component: NotFound},
]

export default routes
