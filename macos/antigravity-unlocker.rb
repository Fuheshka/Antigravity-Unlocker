cask "antigravity-unlocker" do
  version "2.22.0-macos.2"
  sha256 "62ad6a068d2241f5d24c93414c645c9326caec5a2a53ce5f9bb360bb62397700"

  url "https://github.com/Fuheshka/Antigravity-Unlocker/releases/download/v#{version}/Antigravity-Unlocker-macOS.zip"
  name "Antigravity Unlocker"
  desc "Native macOS unlocker tool for Google Antigravity IDE and CLI"
  homepage "https://github.com/Fuheshka/Antigravity-Unlocker"

  depends_on macos: ">= :big_sur"

  app "Antigravity Unlocker.app"
  binary "#{appdir}/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker", target: "ag_unlocker"

  zap trash: [
    "~/Library/Application Support/AGUnlocker",
    "~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist",
  ]
end
