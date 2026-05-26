===description===
!isset short-circuit — foreach loop with nullable accumulator
Reproducer from app-server: !isset($leftPoint) || $item['x'] < $leftPoint
===file===
<?php
/** @param array<array{x: int}> $items */
function test(array $items): void {
    $leftPoint = null;
    foreach ($items as $item) {
        if (!isset($leftPoint) || $item['x'] < $leftPoint) {
            $leftPoint = $item['x'];
        }
    }
}
===expect===
