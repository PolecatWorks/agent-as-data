import os
file_path = "aad-fe-container/src/app/components/traits-registry/traits-registry.component.ts"
import_statement = "import { EntitySidebarListComponent } from '../shared/entity-sidebar-list/entity-sidebar-list.component';\n"
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
