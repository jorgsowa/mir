===file:Helper.php===
<?php
function extractKeys(array $map): array {
    // $filter_value must be optional — calling with one arg must not error
    return array_keys($map);
}

function extractMatchingKeys(array $map, mixed $value): array {
    return array_keys($map, $value);
}
===file:Main.php===
<?php
$keys = extractKeys(['x' => 1, 'y' => 2]);
$matching = extractMatchingKeys(['x' => 1, 'y' => 1, 'z' => 2], 1);
===expect===
