namespace   = "alaydriem"
name        = "peek"
entry       = "alaydriem_peek.dll"
api_version = 2
amml_version = "26.60.6"
entries = [
  # game.ui_render: required, the panel is drawn in the screen view render call
  "ScreenView.render",
  "ScreenView.mPointerLocationPrevious",
  "MinecraftUIRenderContext.mScreenContext",
  "MinecraftUIRenderContext.fillRectangle",
  "MinecraftUIRenderContext.drawRectangle",
  "MinecraftUIRenderContext.drawText",
  "MinecraftUIRenderContext.flushText",
  # game.container_hover: required, which slot the pointer is over, and the shulker box hover text
  "ScreenController.isContainerScreen",
  "ScreenController.dtor",
  "ContainerScreenController.onContainerSlotHovered",
  "ContainerScreenController.onContainerSlotUnhovered",
  "ContainerScreenController.getVisualItemStackImpl",
  "ShulkerBoxBlockItem.appendFormattedHovertext",
  "Item.appendFormattedHovertext",
  # game.item_draw: required, the item pictures, the glint and the durability
  "ItemRenderer.renderGuiItemNew",
  "BaseActorRenderContext.ctor",
  "BaseActorRenderContext.dtor",
  "BaseActorRenderContext.size",
  "ClientInstance.getMinecraftGame",
  "Item.getAnimationFrameFor",
  "Item.isGlint",
  "Item.getMaxDamage",
  "ItemStackBase.getDamageValue",
  "ItemStackBase.isDamaged",
  # game.item_nbt: required, the stored items of the box
  "CompoundTag.nodeVariantIndex",
  "ItemStack.fromTag",
  "Item.mFullName",
  # peek.font: optional, without it the stack counts are not drawn
  "ClientInstance.getFontHandle",
  "FontHandle.dtor",
  "FontHandle.size",
  "Font.getLineLength",
  # game.ui_scroll: optional, the wheel notch that picks a bundle; without it the first bundle shows and the hint is not drawn
  "UIScene.handleDirection",
  "UIScene.mScreenView",
]

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
