# OAuth bridge page

Standalone static file — not part of the Tauri app build. Deploy
`index.html` as-is to `https://tally.valveandmeter.com/` (must be the
domain root, served over HTTPS).

In Wrike's App Console, register the redirect URI as exactly:

```
https://tally.valveandmeter.com/
```

What it does: Wrike's OAuth only accepts a public `https://` redirect
URI, not a loopback address or a custom URL scheme. This page is that
public endpoint — it immediately forwards the `code`/`state`/`error`
query params it receives to `http://127.0.0.1:41820/callback` on the
same machine, where Tally's local login listener picks them up. Nothing
is logged or stored here; it's a pure client-side redirect.

`41820` must match the fixed local port Tally's backend listens on
during login (still to be implemented).
