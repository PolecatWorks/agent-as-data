import re

configs = [
    {
        "file": "aad-fe-container/src/app/components/agent-registry/agent-registry.component.html",
        "items": "getFilteredAgents()",
        "var_name": "agent",
        "id_expr": "agent.id",
        "template_name": "agentCardTemplate",
        "placeholder": "Search agents by name...",
        "empty": "No matching agents found.",
        "click": "selectAgent(agent)",
        "selected_expr": "selectedAgent?.id === agent.id",
        "size": "96",
        "has_custom_header": False,
        "header_title": "AAD",
        "header_icon": "build"
    },
    {
        "file": "aad-fe-container/src/app/components/skills-registry/skills-registry.component.html",
        "items": "getFilteredSkills()",
        "var_name": "skill",
        "id_expr": "skill.id",
        "template_name": "skillCardTemplate",
        "placeholder": "Search skills by name...",
        "empty": "No matching skills found.",
        "click": "selectSkill(skill)",
        "selected_expr": "selectedSkill?.id === skill.id",
        "size": "96",
        "has_custom_header": False,
        "header_title": "AAD",
        "header_icon": "build"
    },
    {
        "file": "aad-fe-container/src/app/components/tool-manager/tool-manager.component.html",
        "items": "getFilteredServers()",
        "var_name": "s",
        "id_expr": "s.id",
        "template_name": "serverCardTemplate",
        "placeholder": "Search tools by name...",
        "empty": "No matching tools found.",
        "click": "selectServer(s)",
        "selected_expr": "selectedServer?.id === s.id",
        "size": "96",
        "has_custom_header": False,
        "header_title": "AAD",
        "header_icon": "build"
    }
]

for c in configs:
    with open(c["file"], "r") as f:
        content = f.read()

    sidebar_start = content.find('<!-- Left Sidebar -->')
    workspace_start = content.find('<!-- Workspace Content')
    if workspace_start == -1:
        workspace_start = content.find('<div class="flex-1 min-w-0', sidebar_start)

    sidebar_html = content[sidebar_start:workspace_start]

    # Extract the item template inner HTML
    # It usually starts with `<div class="group` and ends with the matching `</div>`
    # A robust way is to find `<div class="group` and count divs.
    group_start = sidebar_html.find('<div class="group')
    if group_start == -1:
        print(f"Error: group class not found in {c['file']}")
        continue
    
    # Simple regex to get the content between <div class="group ... and the end of the item loop
    # We can just rely on the fact that the group div is inside the @for or *ngFor
    # Let's find the end of the group div.
    div_count = 0
    i = group_start
    while i < len(sidebar_html):
        if sidebar_html[i:i+4] == "<div":
            div_count += 1
        elif sidebar_html[i:i+5] == "</div":
            div_count -= 1
            if div_count == 0:
                break
        i += 1
    
    group_html = sidebar_html[group_start:i+6]

    # Replace old [ngClass] checks for collapsed state
    group_html = group_html.replace('!isSidebarCollapsed', '!collapsed')
    group_html = group_html.replace('isSidebarCollapsed', 'collapsed')

    new_sidebar = f"""  <!-- Left Sidebar -->
  <app-entity-sidebar-list
    [(isCollapsed)]="isSidebarCollapsed"
    [items]="{c['items']}"
    [itemTemplate]="{c['template_name']}"
    [itemSize]="{c['size']}"
    searchPlaceholder="{c['placeholder']}"
    emptyMessage="{c['empty']}"
    (searchChange)="searchQuery = $event">
  </app-entity-sidebar-list>

  <ng-template #{c['template_name']} let-{c['var_name']} let-collapsed="collapsed">
    {group_html}
  </ng-template>

"""

    content = content[:sidebar_start] + new_sidebar + content[workspace_start:]
    
    with open(c["file"], "w") as f:
        f.write(content)

