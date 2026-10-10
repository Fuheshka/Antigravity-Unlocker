cask "antigravity-unlocker" do
  version "2.21.1-macos.1"
  sha256 "03b027f90a50140bc027d18b91674d28c7d37be3cf9134546990ef3424bd5e9f"

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
