===source===
<?php
class Obj {
    public int $val = 0;
}
function maybeNull(): ?Obj {
    return null;
}
function test(): void {
    $x = maybeNull();
    echo $x->val;
}
===expect===
PossiblyNullPropertyFetch: $x->val
