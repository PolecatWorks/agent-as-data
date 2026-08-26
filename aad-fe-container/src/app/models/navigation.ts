export interface NavMenuItem {
  path: string;
  icon: string;
  label: string;
}

export const APP_NAV_MENU_ITEMS: NavMenuItem[] = [
  { path: '/home', icon: 'home', label: 'Home' },
  { path: '/agents', icon: 'smart_toy', label: 'Agents Registry' },
  { path: '/traits', icon: 'verified', label: 'Traits Registry' },
  { path: '/skills', icon: 'extension', label: 'Skills Registry' },
  { path: '/tools', icon: 'dns', label: 'Tools Registry' },
  { path: '/interactive-testing', icon: 'bug_report', label: 'Testing Studio' },
  { path: '/workbench', icon: 'chat', label: 'Workbench' },
  { path: '/network-visualizer', icon: 'account_tree', label: 'Network Graph' },
  { path: '/refactoring-lab', icon: 'build_circle', label: 'Refactoring Lab' },
  { path: '/knowledge-inspector', icon: 'library_books', label: 'Knowledge Inspector' }
];
