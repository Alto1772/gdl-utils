use crate::common::*;

// 4x3 tiles

test_image_generate!(
    tiling,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["+rows:3"]
);

test_image_generate!(
    tiling_gap_30,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["+rows:3+30"]
);

test_image_generate!(
    tiling_gap_30_percent,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["+rows:3+30%"]
);

// Various Sizes

test_image_generate!(
    tiling_various_sizes,
    ".png",
    map_path(
        &[
            "64.png",
            "64x128.png",
            "128.png",
            "400x200.png",
            "128x64.png",
            "800.png",
            "256x128.png",
            "400x100.png",
            "100.png",
            "128x256.png",
            "256.png",
            "300x100.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["+rows:3"]
);

test_image_generate!(
    tiling_various_sizes_fit_width,
    ".png",
    map_path(
        &[
            "64.png",
            "64x128.png",
            "128.png",
            "400x200.png",
            "128x64.png",
            "800.png",
            "256x128.png",
            "400x100.png",
            "100.png",
            "128x256.png",
            "256.png",
            "300x100.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["+rows:3,fit=w"]
);

test_image_generate!(
    tiling_various_sizes_fit_height,
    ".png",
    map_path(
        &[
            "64.png",
            "64x128.png",
            "128.png",
            "400x200.png",
            "128x64.png",
            "800.png",
            "256x128.png",
            "400x100.png",
            "100.png",
            "128x256.png",
            "256.png",
            "300x100.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["+rows:3,fit=h"]
);

test_image_generate!(
    tiling_various_sizes_fit_height_2,
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
    ["+rows:3,fit=h"]
);

test_image_generate!(
    tiling_various_sizes_rows_single,
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
    ["+rows:1"]
);

test_image_generate!(
    tiling_various_sizes_cols_single,
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
    ["+cols:1"]
);

test_image_generate!(
    tiling_various_sizes_tile_gap_5percent_border_30percent,
    ".png",
    map_path(
        &[
            "64.png",
            "64x128.png",
            "128.png",
            "128x64.png",
            "800.png",
            "256x128.png",
            "100.png",
            "128x256.png",
            "256.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["--border-size", "5%", "+rows:3+30%"]
);

// Remaining tiles alignment

test_image_generate!(
    tiling_rem_align_left,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=left"]
);

test_image_generate!(
    tiling_rem_align_right,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=right"]
);

test_image_generate!(
    tiling_rem_align_center,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=center"]
);

test_image_generate!(
    tiling_rem_align_space_between,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=space-between"]
);

test_image_generate!(
    tiling_rem_align_space_around,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=space-around"]
);

test_image_generate!(
    tiling_rem_align_space_evenly,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 13),
    ["+cols:5,align=space-evenly"]
);

test_image_generate!(
    tiling_rem_rows,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["+rows:5"]
);

test_image_generate!(
    tiling_rem_cols_align_space_between,
    ".png",
    repeat_path("tests/inputs/placeholder/256.png", 12),
    ["+cols:18,align=space-between"]
);
