===description===
`foreach ($f->items as &$v)` mutates a readonly array property's contents
in place, the same way a by-ref call argument (`sort($f->items)`, already
checked) does — but the foreach statement never routed the iterable
expression through the readonly check at all.
===config===
suppress=MissingConstructor,UnusedForeachValue,MixedAssignment
===file===
<?php
class Frozen {
    public readonly array $items;
}

function tick(Frozen $f): void {
    foreach ($f->items as &$v) {
//           ^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Frozen::$items outside of constructor
        $v++;
    }
}
===expect===
