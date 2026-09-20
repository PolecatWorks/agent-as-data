#!/bin/bash
sed -i 's/this.router.navigate(\[`\/${routeType}`, result.id\]);/this.router.navigate(\['\''\/detail'\''\], { queryParams: { id: result.id, type: result.entity_type } });/g' aad-fe-container/src/app/pages/semantic-search/semantic-search.component.ts
