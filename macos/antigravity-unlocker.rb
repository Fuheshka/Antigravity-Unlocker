cask "antigravity-unlocker" do
  version "2.17.0.3-macos.1"
  sha256 "a361e829ee0b83935cb3373cc30dff4464464f57a0523c599860443bcd4f659f"

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
