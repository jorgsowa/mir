===description===
array_map with multiple array arguments
===file===
<?php

$keys = ['a', 'b'];
$counts = [1, 2];

$labels = array_map(
    static fn(string $k, int $c): string => "{$k}({$c})",
    $keys,
    $counts,
);

===expect===
