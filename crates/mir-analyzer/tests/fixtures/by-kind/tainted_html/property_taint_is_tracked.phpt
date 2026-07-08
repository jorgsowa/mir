===description===
Taint via a single-hop instance property assignment must be tracked —
assigning a tainted value to $obj->prop and later reading $obj->prop was
previously never recognized as tainted (is_expr_tainted had no
PropertyAccess arm at all).
===config===
suppress=MixedAssignment,MixedArgument,MixedArrayAccess,MissingPropertyType
===file===
<?php
class Box {
    public $value;
}
function test(): void {
    $b = new Box();
    $b->value = $_GET['x'];
    echo $b->value;
}
===expect===
TaintedHtml@8:4-8:19: Tainted HTML output — possible XSS
