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
    /** @mir-check $x is Obj|null */
    echo $x->val;
//       ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $val on possibly null value
}
===expect===
