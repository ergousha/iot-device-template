# keys/

The firmware-signing **public** keys, `<id>-a.pub.pem` (signs releases, from
Environment `release`) and `<id>-b.pub.pem` (the offline backup).
`ergo product init` writes them and names them in `platform.toml`
`[signing] keys`; every image accepts both. Never a private key here.
