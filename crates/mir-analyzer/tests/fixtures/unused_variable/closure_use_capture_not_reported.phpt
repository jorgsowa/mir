===file===
<?php
function foo(): array {
    $config = ['multiplier' => 2];
    $items = [1, 2, 3];
    return array_map(function($item) use ($config) {
        return $item * $config['multiplier'];
    }, $items);
}
===expect===
