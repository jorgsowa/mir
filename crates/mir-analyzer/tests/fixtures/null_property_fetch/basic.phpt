===file===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
}
===expect===
NullPropertyFetch: Cannot access property $prop on null
