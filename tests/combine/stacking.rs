use crate::common::*;

// Horizontal stacking

test_image_generate!(
    horizontal,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal"]
);

test_image_generate!(
    gap,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--gap", "10"]
);

test_image_generate!(
    gap_percent,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--gap", "10%"]
);

test_image_generate!(
    border,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--border-size", "50"]
);

test_image_generate!(
    border_percent,
    ".jpg",
    listglob("tests/inputs/placeholder/1200x800-*.jpg"),
    ["-O", "horizontal", "--border-size", "12.5%"]
);

// Various Sizes

test_image_generate!(
    hstack_various_sizes,
    ".png",
    map_path(
        &[
            "64.png",
            "128.png",
            "64.png",
            "128x64.png",
            "64.png",
            "64x128.png",
        ],
        "tests/inputs/placeholder/"
    ),
    ["-O", "horizontal"]
);

test_image_generate!(
    vstack_various_sizes,
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
    ["-O", "vertical"]
);
