-- In /home/marc/dev/esp-led/.nvim.lua

print("NEOVIM: Loading LOCAL Docker + ESP-IDF config...")

local lspconfig = require('lspconfig')
local host_root = vim.fn.getcwd()

-- SET THIS to your container mount path
local container_root = "/app/rust_project" -- (Or /workdir, etc.)

lspconfig.rust_analyzer.setup{
  cmd = {"docker", "exec", "-i", "frosty_taussig", "rust-analyzer"},

  settings = {
    ["rust-analyzer"] = {
      -- 1. The path mapping (you already have this)
      remapPathPrefix = {
        [host_root] = container_root
      },

      -- 2. The *new* critical settings for esp-idf
      cargo = {
        -- This tells r-a to load info from a cargo check build
        loadOutDirsFromCheck = true,
        -- This tells r-a to ONLY check the target in .cargo/config.toml
        allTargets = false,
      },
      check = {
        -- This also prevents checking for your host (x86_64)
        allTargets = false,
      }
    }
  }
}
