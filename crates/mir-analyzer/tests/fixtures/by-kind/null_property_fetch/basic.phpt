===description===
Basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
}
===expect===
NullPropertyFetch@4:10-4:18: Cannot access property $prop on null
