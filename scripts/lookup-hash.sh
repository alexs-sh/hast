#!/bin/bash

SERVER=${SERVER:-"127.0.0.1:8888"}
HASH=${HASH:-"HASH"}
WDIR=${WDIR:-"/tmp/hast/lookup-hash/"}
PRETTY=${PRETTY:-0}
RESPONSE=$WDIR/response"$$".json


fatal()
{
    echo "fatal:$1"
    exit 1
}

make_request()
{
    echo "{"
    echo "  \"hashes\" : [\"$HASH\"]"
    echo "}"
}

init()
{
    if [ "$HASH" = "" ]; then
        fatal "hash is empty"
    fi
    mkdir -p "$WDIR" 2>/dev/null
}

send_request()
{
    tmp="$WDIR"/request"$$".json
    make_request > "$tmp"
    curl -s -d "@${tmp}" -X GET -H "Content-Type: application/json" "http://${SERVER}/lookup" > "$RESPONSE"
    rm "$tmp"
}

print_response()
{
    if [ "$PRETTY" -eq 1 ]; then
        jq . < "$RESPONSE"
    else
        cat "$RESPONSE"
    fi
}

init
send_request
print_response

