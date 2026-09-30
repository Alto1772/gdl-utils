use crate::common::*;

test_image_generate!(
    heavy_tiling,
    ".jpg",
    listglob("tests/inputs/picsum/*-3000x5000.jpg"),
    ["+rows:3"]
);

test_image_generate!(
    heavy_tiling_resize_width_1000,
    ".jpg",
    listglob("tests/inputs/picsum/*-3000x5000.jpg"),
    ["--resize", "1000", "+rows:3"]
);

test_image_generate!(
    heavy_tiling_resize_1000x1000,
    ".jpg",
    listglob("tests/inputs/picsum/*-3000x5000.jpg"),
    ["--resize", "1000x1000", "+rows:3"]
);
