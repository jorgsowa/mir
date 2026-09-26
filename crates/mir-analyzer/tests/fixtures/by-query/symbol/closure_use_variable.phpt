===description===
A variable captured by a closure's `use` resolves inside the closure.
===cursor===
symbol
===file===
<?php
function f(): void {
    $prefix = 'id-';
    $g = function (string $s) use ($prefix): string {
        return $pre<CURSOR>fix . $s;
    };
}
===expect===
kind: variable $prefix
type: "id-"
