#!/bin/bash
awk '
/^pub mod search;/ { found=1 }
/pub fn app_router/ {
    if (!found) {
        print "pub mod search;"
    }
}
/api_routes = Router::new/ {
    print
    print "        .nest(\"/v1/search\", search::router())"
    next
}
{ print }
' aad-be-container/src/webserver/mod.rs > aad-be-container/src/webserver/mod.rs.tmp
mv aad-be-container/src/webserver/mod.rs.tmp aad-be-container/src/webserver/mod.rs
