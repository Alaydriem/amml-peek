use alaydriem_peek::traits::Canvas;
use peek_core::structs::{Rect, Rgb};

use super::TrackedStack;

#[derive(Debug, Clone, PartialEq)]
pub enum CanvasOp {
    Fill { area: Rect, color: Rgb, alpha: f32 },
    Outline { area: Rect, color: Rgb, alpha: f32, thickness: i32 },
    Item { name: String, icon: Rect, foil: bool, alpha: f32, z_order: i32 },
    Count { text: String, cell: Rect },
    Label { text: String, area: Rect },
    FlushText,
}

// a canvas that keeps every call in order
#[derive(Default)]
pub struct RecordingCanvas {
    pub ops: Vec<CanvasOp>,
}

impl Canvas for RecordingCanvas {
    type Stack = TrackedStack;

    fn fill(&mut self, area: Rect, color: Rgb, alpha: f32) {
        self.ops.push(CanvasOp::Fill { area, color, alpha });
    }

    fn outline(&mut self, area: Rect, color: Rgb, alpha: f32, thickness: i32) {
        self.ops.push(CanvasOp::Outline { area, color, alpha, thickness });
    }

    fn item(&mut self, stack: &TrackedStack, icon: Rect, foil: bool, alpha: f32, z_order: i32) {
        self.ops.push(CanvasOp::Item { name: stack.name.clone(), icon, foil, alpha, z_order });
    }

    fn count(&mut self, text: &str, cell: Rect) {
        self.ops.push(CanvasOp::Count { text: text.to_string(), cell });
    }

    fn label(&mut self, text: &str, area: Rect) {
        self.ops.push(CanvasOp::Label { text: text.to_string(), area });
    }

    fn flush_text(&mut self) {
        self.ops.push(CanvasOp::FlushText);
    }
}
