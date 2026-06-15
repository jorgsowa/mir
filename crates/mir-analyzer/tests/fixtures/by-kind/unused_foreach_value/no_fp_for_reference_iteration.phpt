===description===
foreach (&$val) where $val is modified in the loop body should not emit UnusedForeachValue — the write goes back to the original array through the reference
===file===
<?php
function transform(string $s): string { return strtoupper($s); }

$items = ['a', 'b', 'c'];

// Should NOT report UnusedForeachValue — writing to &$item modifies $items.
foreach ($items as &$item) {
    $item = transform($item);
}
===expect===
