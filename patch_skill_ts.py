with open('aad-fe-container/src/app/components/skills-registry/skills-registry.component.ts', 'r') as f:
    content = f.read()

# Add missing fields that were lost when the file was restored implicitly somewhere or git checked out

search_query = "  traitSearchQuery = '';"
if "usesTraitSearchQuery = '';" not in content:
    content = content.replace(search_query, search_query + '\n  usesTraitSearchQuery = \'\';')

traits_filter_method = """  get filteredTraitsCatalog(): string[] {
    const attached = this.skillForm.implements_traits || [];
    return this.traitsCatalog.filter(t =>
      !attached.includes(t) &&
      t.toLowerCase().includes(this.traitSearchQuery.toLowerCase())
    );
  }"""

uses_traits_filter_method = """  get filteredUsesTraitsCatalog(): string[] {
    const attached = this.skillForm.uses_traits || [];
    return this.traitsCatalog.filter(t =>
      !attached.includes(t) &&
      t.toLowerCase().includes(this.usesTraitSearchQuery.toLowerCase())
    );
  }"""

if "filteredUsesTraitsCatalog" not in content:
    content = content.replace(traits_filter_method, traits_filter_method + '\n\n' + uses_traits_filter_method)


attach_trait = """  attachTraitFromCatalog(trait: string) {
    if (!this.skillForm.implements_traits) this.skillForm.implements_traits = [];
    if (!this.skillForm.implements_traits.includes(trait)) {
      this.skillForm.implements_traits.push(trait);
    }
  }"""

attach_uses_trait = """  attachUsesTraitFromCatalog(trait: string) {
    if (!this.skillForm.uses_traits) this.skillForm.uses_traits = [];
    if (!this.skillForm.uses_traits.includes(trait)) {
      this.skillForm.uses_traits.push(trait);
    }
  }"""

if "attachUsesTraitFromCatalog" not in content:
    content = content.replace(attach_trait, attach_trait + '\n\n' + attach_uses_trait)


remove_trait = """  removeTrait(trait: string) {
    if (this.skillForm.implements_traits) {
      this.skillForm.implements_traits = this.skillForm.implements_traits.filter((t: string) => t !== trait);
    }
  }"""

remove_uses_trait = """  removeUsesTrait(trait: string) {
    if (this.skillForm.uses_traits) {
      this.skillForm.uses_traits = this.skillForm.uses_traits.filter((t: string) => t !== trait);
    }
  }"""

if "removeUsesTrait" not in content:
    content = content.replace(remove_trait, remove_trait + '\n\n' + remove_uses_trait)

with open('aad-fe-container/src/app/components/skills-registry/skills-registry.component.ts', 'w') as f:
    f.write(content)
