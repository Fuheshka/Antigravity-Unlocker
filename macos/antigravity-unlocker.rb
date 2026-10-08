cask "antigravity-unlocker" do
  version "2.19.1-macos.1"
  sha256 "20ba4038e10290ca7f826426c6cd9024e085bd0653351fdcef9339ddb8615cd3"

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
