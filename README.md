### About

Dummy hash storage. It allows saving / restoring / searching hashes of files
using JSON over HTTP.

[![Build Status](https://gitlab.com/alexssh/hast/badges/master/pipeline.svg)](https://gitlab.com/alexssh/hast/-/commits/master)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

### Example

Running server

```
 cargo run
```

Generate hashes for /tmp directory, send them to a server, and receive
information about the first hashed file.

```
find /tmp  -type f -exec sha512sum {} \; > /tmp/hash-1
FILE=/tmp/hash-1 ID=hash-1 ./scripts/send-report.sh
hash=$(head -1 /tmp/hash-1 | cut -f 1 -d ' ')
PRETTY=1 HASH=$hash ./scripts/lookup-hash.sh
{
  "records": [
    {
      "id": "hash-1",
      "host": "uber-host",
      "timestamp": "Wed Sep 21 08:23:10 AM CEST 2022"
    }
  ]
}
```

