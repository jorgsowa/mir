===description===
PossiblyNullPropertyFetch fires when fetching a property on a nullable
parameter without a null guard.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(?Obj $obj): void {
    echo $obj->name;
//       ^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $name on possibly null value
}
===expect===
