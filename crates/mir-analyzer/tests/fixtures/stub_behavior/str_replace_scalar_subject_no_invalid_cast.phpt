===description===
str_replace / str_ireplace with non-array scalar subject returns string — no InvalidCast
===file===
<?php
// Integer passed via PHP int (coerced to string by PHP): flipped conditional
// ensures str_replace returns string when subject is not an array.
function replaceInInt(int $n): float {
    $result = str_replace(',', '.', (string) $n);
    return (float) $result;
}

// Subject from explode element (string) — most common real-world shape
function replaceInExplode(string $s): float {
    return (float) str_replace(')', '', explode(',', $s)[3]);
}

// str_ireplace with explode element
function replaceInExplodeI(string $s): int {
    return (int) str_ireplace('rgb(', '', explode(',', $s)[0]);
}
===expect===
