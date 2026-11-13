# Security

Report a vulnerability privately through GitHub's **Report a vulnerability**
(Security tab), not in a public issue.

This repository is public on purpose: images, manifests and SBOMs carry no
secrets. The claim credential, device keys, Wi-Fi passwords and the signing
key never enter Git, CI logs, caches, artifacts or images; the release
workflow's image scan enforces it. See PLATFORM.md (iot-device-sdk) §3, §11.
