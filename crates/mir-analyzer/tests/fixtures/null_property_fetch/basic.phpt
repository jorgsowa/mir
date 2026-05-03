===description===
basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
}
===expect===
NullPropertyFetch@4:9: Cannot access property $prop on null
===ignore===
TODO
