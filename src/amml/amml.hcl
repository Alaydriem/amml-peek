namespace    = "alaydriem"
name         = "peek"
entry        = "alaydriem_peek.dll"
api_version  = 3

uses {
  feature "preview" {
    event "ui.screen_render" {
      version  = 1
      required = true
    }
    event "ui.container_slot_hover" {
      version  = 1
      required = true
    }
    event "ui.container_slot_unhover" {
      version  = 1
      required = true
    }
    event "ui.screen_closing" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "registry.provider_gone" {
      version  = 1
      required = true
    }
    event "mods.loaded" {
      version  = 1
      required = true
    }
    area "ui.draw" {
      version  = 2
      required = true
    }
    area "ui.screen" {
      version  = 2
      required = true
    }
    area "inventory.read" {
      version  = 4
      required = true
    }
    event "input.lease_suspended" {
      version  = 1
      required = false
    }
    event "input.lease_resumed" {
      version  = 1
      required = false
    }
    event "input.lease_lost" {
      version  = 1
      required = false
    }
    event "input.injected_released" {
      version  = 2
      required = false
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = false
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = false
    }
    area "input.lease" {
      version  = 1
      required = false
    }
  }
  feature "counts" {
    area "ui.text" {
      version  = 1
      required = true
    }
  }
  feature "bundle_pick" {
    event "input.ui_scroll" {
      version  = 1
      required = true
    }
    event "input.key" {
      version  = 1
      required = true
    }
  }
  feature "hovertext" {
    event "ui.item_hovertext" {
      version  = 1
      required = true
    }
  }
}

listing {
  title       = "Peek"
  summary     = "See inside a shulker box by hovering over it."
  description = <<-EOT
    Peek inside shulker boxes from any in-game container to preview their contents, with support for durability, stack sizes, and even bundles.
  EOT
  authors     = ["Charles R. Portwood II"]
  license     = "BSD-3-Clause"
  source      = "https://github.com/alaydriem/amml-peek"
  homepage    = "https://github.com/alaydriem/amml-peek#readme"
  tags        = ["shulker", "inventory", "preview", "client", "utility"]
  icon        = "icon.png"
}
