#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
identity="8bitEmu Local"
keychain="${HOME}/Library/Keychains/8bitemu-signing.keychain-db"
dist="${root}/dist"
app="${dist}/8bitEmu.app"
dmg="${dist}/8bitEmu.dmg"
macos="${app}/Contents/MacOS"
shipped="$("${root}/scripts/build-release.sh")"
binary="${root}/${shipped}"

rm -rf "${app}"
mkdir -p "${macos}"
cp "${binary}" "${macos}/8bitEmu"
chmod +x "${macos}/8bitEmu"
cat > "${app}/Contents/Info.plist" <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>8bitEmu</string>
  <key>CFBundleIdentifier</key>
  <string>com.tenprintsoftware.8bitEmu</string>
  <key>CFBundleName</key>
  <string>8bitEmu</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.9.15</string>
  <key>LSMinimumSystemVersion</key>
  <string>13.0</string>
</dict>
</plist>
EOF
codesign --force --sign "${identity}" --keychain "${keychain}" "${app}"
rm -f "${dmg}"
hdiutil create -volname 8bitEmu -srcfolder "${app}" -ov -format UDZO "${dmg}"
echo "OK: ${dmg}"
