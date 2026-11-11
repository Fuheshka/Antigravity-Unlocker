cask "antigravity-unlocker" do
  version "2.29.0-macos.1"
  sha256 "5cea2c5f832b2746b7e6154f896df44fbc70e59240b95376ff0947cf76af6336"

  url "https://github.com/Fuheshka/Antigravity-Unlocker/releases/download/v#{version}/Antigravity-Unlocker-macOS.zip"
  name "Antigravity Unlocker"
  desc "Native macOS unlocker tool for Google Antigravity IDE and CLI"
  homepage "https://github.com/Fuheshka/Antigravity-Unlocker"

  depends_on macos: :big_sur

  app "Antigravity Unlocker.app"
  binary "#{appdir}/Antigravity Unlocker.app/Contents/MacOS/ag_unlocker", target: "ag_unlocker"

  zap trash: [
    "~/Library/Application Support/AGUnlocker",
    "~/.local/share/agunlocker",
    "~/Library/LaunchAgents/com.antigravity.unlocker.proxy.plist",
    "~/Library/LaunchAgents/com.agunlocker.proxy.plist",
    "~/Library/LaunchAgents/com.agunlocker.watchdog.plist",
    "~/Library/LaunchAgents/com.agunlocker.env.plist",
  ]
end
