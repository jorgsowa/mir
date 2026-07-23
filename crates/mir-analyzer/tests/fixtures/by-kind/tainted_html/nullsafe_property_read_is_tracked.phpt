===description===
`is_expr_tainted` had no arm for `NullsafePropertyAccess` -- only the plain
`PropertyAccess` variant was matched, so a tainted property read through
`?->` fell to the catch-all `_ => false`.
===config===
suppress=MixedAssignment,MissingConstructor,MissingPropertyType,MixedArrayAccess
===file===
<?php
class User {
    public $name;
}
function test(?User $u): void {
    $u->name = $_GET['name'];
    echo $u?->name;
}
===expect===
TaintedHtml@7:4-7:19: Tainted HTML output — possible XSS
