===description===
$obj?->prop after a null-check on $obj->prop must use the narrowed (non-null) type, matching the plain $obj->prop path.
===file===
<?php
class Box { public ?string $val = null; }
function f(Box $b): void {
    if ($b->val !== null) {
        $x = $b->val;
        /** @mir-check $x is string */
        echo $x;
        $y = $b?->val;
        /** @mir-check $y is string */
        echo $y;
    }
}
===expect===
