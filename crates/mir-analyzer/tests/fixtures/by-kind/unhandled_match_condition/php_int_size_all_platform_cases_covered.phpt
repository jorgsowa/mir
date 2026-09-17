===description===
PHP_INT_SIZE has the precise runtime domain 4|8, so a match covering both
supported integer widths is exhaustive without a default arm.
===file===
<?php
function decimalChunks(): int {
    return match (PHP_INT_SIZE) {
        4 => 9,
        8 => 18,
    };
}
===expect===
