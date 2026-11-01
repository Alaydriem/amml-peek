namespace   = "alaydriem"
name        = "peek"
entry       = "alaydriem_peek.dll"
api_version = 2
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
