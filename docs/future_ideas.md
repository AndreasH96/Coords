# Future Ideas

## Password/passphrase key derivation
Instead of requiring a GPS coordinate as the key, let users pass a password that deterministically generates an origin coordinate (e.g. via PBKDF2 → lat/lon). Easier to remember than raw coordinates.

## Progress bar
Encoding is sequential (chained), so large files take a while. A terminal progress indicator would help UX.

## Export to GPX/KML
Output coordinates in standard GPS formats that can be loaded into Google Earth or GPS apps.

