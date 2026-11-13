# web/

An optional web app for the product's users. Rules (PLATFORM.md §10):

- its own origin: never the platform portal's, and never
  `<owner>.github.io` shared with other sites that could read its tokens;
- commands go to the devices through the product's shadow desired state;
- no auto-polling: refresh on a user action, or at most every 60 s while the
  tab is visible;
- end-user sign-up, if any, uses a user pool the platform's identity pool
  never trusts.

Delete this directory if the product has no web app.
