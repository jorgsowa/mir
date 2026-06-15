===description===
Basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
}
===expect===
NullPropertyFetch@4:9-4:17: Cannot access property $prop on null
