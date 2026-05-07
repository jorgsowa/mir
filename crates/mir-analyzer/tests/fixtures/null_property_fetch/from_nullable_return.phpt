===description===
from nullable return
===file===
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
PossiblyNullPropertyFetch@10:9: Cannot access property $val on possibly null value
