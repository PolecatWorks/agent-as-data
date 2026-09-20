#!/bin/bash
awk '
/export const routes: Routes = \[/ {
    print
    print "  { path: '\''semantic-search'\'', loadComponent: () => import('\''./pages/semantic-search/semantic-search.component'\'').then(m => m.SemanticSearchComponent) },"
    next
}
{ print }
' aad-fe-container/src/app/app.routes.ts > aad-fe-container/src/app/app.routes.ts.tmp
mv aad-fe-container/src/app/app.routes.ts.tmp aad-fe-container/src/app/app.routes.ts
