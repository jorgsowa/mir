===description===
PossiblyNullPropertyFetch does NOT fire when a null check guards the property
fetch.
===file===
<?php
class Obj { public string $name = 'x'; }
function test(?Obj $obj): void {
    if ($obj !== null) {
        echo $obj->name;
    }
}
===expect===
