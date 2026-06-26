===description===
PossiblyNullPropertyFetch fires when fetching a property on a nullable
parameter without a null guard.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(?Obj $obj): void {
    echo $obj->name;
}
===expect===
PossiblyNullPropertyFetch@4:9-4:19: Cannot access property $name on possibly null value
