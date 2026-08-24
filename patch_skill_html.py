with open('aad-fe-container/src/app/components/skills-registry/skills-registry.component.html', 'r') as f:
    content = f.read()

used_traits_section = """
                  <!-- Section 4: Used Trait Contracts -->
                  <div class="p-5 bg-indigo-50/50 rounded-xl border border-indigo-200/80 space-y-4">
                    <div>
                      <h4 class="text-sm font-bold text-indigo-950 m-0 flex items-center gap-2">🔗 Used Trait Contracts</h4>
                    </div>

                    @if (isEditing) {
                      <div class="space-y-2">
                        <mat-form-field appearance="outline" class="w-full" matTooltip="Type to filter registered trait interface contracts to depend on">
                          <mat-label>Search Registered Used Trait Contracts...</mat-label>
                          <input matInput [(ngModel)]="usesTraitSearchQuery" placeholder="Filter traits e.g. SecurityAuditor">
                        </mat-form-field>

                        <div class="flex flex-wrap gap-2 p-3 bg-white rounded-xl border border-slate-200 max-h-36 overflow-y-auto">
                          @for (t of filteredUsesTraitsCatalog; track t) {
                            <button mat-stroked-button color="primary" class="!rounded-lg !text-xs" [matTooltip]="getTraitDescription(t)" (click)="attachUsesTraitFromCatalog(t)">
                              + {{ t }}
                            </button>
                          } @empty {
                            <span class="text-xs text-slate-400 italic">No traits available to attach.</span>
                          }
                        </div>
                      </div>
                    }

                    <div class="space-y-2 pt-2 border-t border-slate-200">
                      <div class="flex flex-wrap gap-2">
                        @for (trait of skillForm.uses_traits || []; track trait) {
                          <span class="px-3 py-1 bg-indigo-100 text-indigo-800 font-bold text-xs rounded-xl flex items-center gap-2 border border-indigo-200" [matTooltip]="getTraitDescription(trait)">
                            🔗 {{ trait }}
                            @if (isEditing) {
                              <button class="text-indigo-600 hover:text-red-600 font-bold border-none bg-transparent cursor-pointer text-sm" (click)="removeUsesTrait(trait)">×</button>
                            }
                          </span>
                        } @empty {
                          <span class="text-xs text-slate-400 italic">No used traits attached.</span>
                        }
                      </div>
                    </div>
                  </div>
"""

to_replace = """                </div>
              </mat-tab>

              <!-- TAB 2: Interface -->"""

content = content.replace(to_replace, used_traits_section + '\n\n' + to_replace)

with open('aad-fe-container/src/app/components/skills-registry/skills-registry.component.html', 'w') as f:
    f.write(content)
