#!/bin/bash
sed -i 's/vec!\["0.0"; 1536\]/vec!\["0.1"; 1536\]/g' aad-be-container/src/webserver/search.rs
