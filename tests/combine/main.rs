mod background;
mod common;
mod heavy;
mod resize;
mod stacking;
mod tiling;

use crate::common::*;

test_image_generate!(
    defaults,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg")
);
