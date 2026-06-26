===description===
PossiblyNullPropertyFetch does NOT fire when a throw-based guard narrows the
type to non-null before the property fetch.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(?Obj $obj): void {
    if ($obj === null) {
        throw new \InvalidArgumentException('obj required');
    }
    echo $obj->name;
}
===expect===
