mod squares;

use super::*;
use squares::*;

#[component]
pub fn Board(flipped: ReadOnlySignal<bool>) -> Element {
    rsx! {
        svg {
            width: "100%",
            height: "100%",
            view_box: "0 0 720 720",
        }
    }
}
