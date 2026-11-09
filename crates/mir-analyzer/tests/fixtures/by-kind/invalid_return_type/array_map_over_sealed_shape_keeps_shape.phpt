===description===
array_map over a sealed shape returns the same keys with the callback's return type
===file===
<?php
/** @return array{0: float, 1: float, 2: float, 3: float} */
function bbox(): array {
    return array_map(fn (string $edge): float => 1.0, ['west', 'south', 'east', 'north']);
}
/** @return array{lat: int, lon: int} */
function coords(): array {
    return array_map(fn (string $v): int => (int) $v, ['lat' => '1', 'lon' => '2']);
}
