#!/bin/bash

SERVER=${SERVER:-"127.0.0.1:8888"}
FILE=${FILE:-"report"}
WDIR=${WDIR:-"/tmp/hast/send-report/"}
ID=${ID:-""}
HOST=${HOST:-"$(hostname)"}
TIMESTAMP=${TIMESTAMP:-"$(date)"}


fatal()
{
    echo "fatal:$1"
    exit 1
}

make_report()
{
    payload=$(base64 -w 0 "$FILE")
    res=$?
    if [ "$res" -ne 0 ]; then
        fatal "base64 encoding failed"
    fi

    echo "{"
    echo "  \"info\" : {"
    echo "      \"id\" : \"$ID\","
    echo "      \"host\" : \"$HOST\","
    echo "      \"timestamp\" : \"$TIMESTAMP\""
    echo "  },"
    echo "  \"payload\" : {"
    echo "      \"data\" : \"$payload\""
    echo "  }"
    echo "}"
}

init()
{
    if [ "$ID" = "" ]; then
        fatal "ID is empty"
    fi

    if [ ! -f "$FILE" ]; then
        fatal "$FILE is missing"
    fi

    mkdir -p "$WDIR" 2>/dev/null
}

send_report()
{
    tmp="$WDIR"/report"$$".json
    make_report > "$tmp"
    curl -d "@${tmp}" -X POST -H "Content-Type: application/json" "http://${SERVER}/insert"
    rm "$tmp"
}

init
send_report

