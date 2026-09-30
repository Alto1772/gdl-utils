use crate::common::*;

test_image_generate!(
    horizontal_stack_resize_width_1000,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--resize", "1000"]
);

test_image_generate!(
    horizontal_stack_resize_height_1000,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--resize", "x1000"]
);

test_image_generate!(
    vstack_various_sizes_resize_2000,
    ".png",
    map_path(
        &[
            "200x100.png",
            "300x100.png",
            "400x100.png",
            "400x200.png",
            "800x500.png",
            "256x128.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["-O", "vertical", "--resize", "2000"]
);

test_image_generate!(
    tiling_resize_bound_1000x1000,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["--resize", "1000x1000", "+rows:3+30"]
);

test_image_generate!(
    tiling_resize_shrink_width_1000,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["--resize", "1000>", "+rows:3+30"]
);

test_image_generate!(
    tiling_resize_enlarge_height_2000,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["--resize", "x2000<", "+rows:3+30"]
);

test_image_generate!(
    tiling_resize_enlarge_height_800_no_effect,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["--resize", "x800<", "+rows:3+30"]
);
