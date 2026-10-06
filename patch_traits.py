import re

filepath = "aad-fe-container/src/app/components/traits-registry/traits-registry.component.html"

with open(filepath, "r") as f:
    content = f.read()

sidebar_start = content.find('<!-- Left Sidebar -->')
sidebar_end_comment = content.find('<!-- Right Area')
# Actually, we can just replace everything from <!-- Left Sidebar --> up to the end of the sidebar wrapper.
# In the newly restructured html, the sidebar is inside `<div class="flex-1 flex flex-row min-h-0 overflow-hidden relative">`
# and there's no `<!-- Right Area -->`.
# The sidebar wrapper is `<div [ngClass]="isSidebarCollapsed ? 'w-16' : 'w-72'" ...>`
# Let's find the end of the sidebar wrapper. It ends where the Workspace Content wrapper begins.
workspace_start = content.find('<!-- Workspace Content')

sidebar_html = content[sidebar_start:workspace_start]

# We want to replace it with <app-entity-sidebar-list>
new_sidebar = """  <!-- Left Sidebar -->
  <app-entity-sidebar-list
    [(isCollapsed)]="isSidebarCollapsed"
    [items]="filteredTraitContracts"
    [itemTemplate]="traitCardTemplate"
    [itemSize]="110"
    searchPlaceholder="Search traits..."
    emptyMessage="No traits found."
    (searchChange)="searchQuery = $event">
  </app-entity-sidebar-list>

  <ng-template #traitCardTemplate let-trait let-collapsed="collapsed">
    <div class="group p-3 rounded-xl border transition-all cursor-pointer bg-white"
         [ngClass]="selectedTraitContract?.id === trait.id ? 'border-indigo-500 shadow-sm ring-1 ring-indigo-500/20' : 'border-slate-200 hover:border-slate-300'"
         (click)="selectTraitContract(trait)"
         [matTooltip]="trait.name"
         matTooltipPosition="right"
         [matTooltipDisabled]="!collapsed">

      <!-- Expanded View -->
      <div class="flex flex-col w-full" *ngIf="!collapsed">
        <div class="flex justify-between items-start">
          <div class="flex items-start gap-3 overflow-hidden flex-1">
            <mat-icon class="shrink-0 text-slate-400 mt-0.5"
                      [ngClass]="{'text-indigo-600': selectedTraitContract?.id === trait.id}">
              verified
            </mat-icon>
            <div class="flex-1 min-w-0">
              <h5 class="text-sm font-bold text-slate-900 m-0 truncate leading-tight mt-0.5">{{ trait.name }}</h5>
            </div>
          </div>

          <div class="flex items-center gap-1 ml-2 shrink-0">
            <span class="px-2.5 py-0.5 bg-emerald-100 text-emerald-800 text-[10px] font-bold rounded-md">
              v{{ trait.version }}
            </span>
          </div>
        </div>

        <!-- Description -->
        <p *ngIf="trait.description" class="text-xs text-slate-500 m-0 mt-2 line-clamp-2 leading-relaxed">
          {{ trait.description }}
        </p>

        <!-- Tags -->
        <div *ngIf="trait.tags && trait.tags.length > 0" class="flex flex-wrap gap-1 pt-2">
          <span *ngFor="let tag of trait.tags" class="px-2 py-0.5 bg-slate-100 text-slate-600 text-[10px] font-mono rounded">
            #{{ tag }}
          </span>
        </div>
      </div>

      <!-- Collapsed View -->
      <div class="flex justify-center items-center" *ngIf="collapsed">
         <mat-icon class="text-slate-400 transition-colors"
                  [ngClass]="{'text-indigo-600': selectedTraitContract?.id === trait.id}">
          verified
        </mat-icon>
      </div>
    </div>
  </ng-template>

"""

content = content[:sidebar_start] + new_sidebar + content[workspace_start:]

with open(filepath, "w") as f:
    f.write(content)

