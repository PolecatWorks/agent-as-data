import os

components = [
    "aad-fe-container/src/app/components/agent-registry/agent-registry.component.ts",
    "aad-fe-container/src/app/components/skills-registry/skills-registry.component.ts",
    "aad-fe-container/src/app/components/tool-manager/tool-manager.component.ts",
    "aad-fe-container/src/app/components/workbench/workbench.component.ts",
    "aad-fe-container/src/app/components/interactive-testing/interactive-testing.component.ts",
    "aad-fe-container/src/app/components/knowledge-inspector/knowledge-inspector.component.ts",
    "aad-fe-container/src/app/components/network-visualizer/network-visualizer.component.ts"
]

import_statement = "import { EntitySidebarListComponent } from '../shared/entity-sidebar-list/entity-sidebar-list.component';\n"

for file_path in components:
    with open(file_path, "r") as f:
        content = f.read()

    if "EntitySidebarListComponent" not in content:
        concept_idx = content.find("import { TopNavbarComponent")
        if concept_idx == -1:
            concept_idx = content.find("import {")
        insert_idx = content.find(";", concept_idx) + 1
        content = content[:insert_idx] + "\n" + import_statement + content[insert_idx:]

    if "EntitySidebarListComponent," not in content:
        imports_arr_idx = content.find("imports: [")
        if imports_arr_idx != -1:
            insert_idx = imports_arr_idx + len("imports: [")
            content = content[:insert_idx] + "\n    EntitySidebarListComponent," + content[insert_idx:]

    with open(file_path, "w") as f:
        f.write(content)
