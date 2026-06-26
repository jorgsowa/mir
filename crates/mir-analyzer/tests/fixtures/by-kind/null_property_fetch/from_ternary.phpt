===description===
A ternary that may produce null yields a possibly-null type, reporting
PossiblyNullPropertyFetch instead of NullPropertyFetch.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(bool $flag): void {
    $x = $flag ? new Obj() : null;
    echo $x->name;
}
===expect===
PossiblyNullPropertyFetch@5:9-5:17: Cannot access property $name on possibly null value
