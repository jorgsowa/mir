===source===
<?php
function foo(): array {
    $factor = 2;
    $items = [1, 2, 3];
    return array_map(fn($item) => $item * $factor, $items);
}
===expect===
