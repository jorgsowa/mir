===description===
A key assigned on only some paths is possibly absent, not nonexistent.
===file===
<?php
function conditional_append(): int {
    $o = [];
    if (rand() > 1) { $o['left'] = 1; }
    /** @mir-check $o is array{'left': 1}|array{} */
    return $o['left'];
}

function ternary_shapes(bool $c): void {
    $opts = $c ? ['key' => 'a', 'type' => 'int'] : ['key' => 'b'];
    echo $opts['key'];
    echo $opts['type'];
}

function nested_branches(int $n): string {
    $o = ['base' => 'x'];
    if ($n > 1) {
        $o['extra'] = 'y';
    } elseif ($n > 0) {
        $o['extra'] = 'z';
    }
    return $o['extra'];
}
===expect===
