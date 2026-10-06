===description===
Basic
===file===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
//       ^^^^^^^^ NullPropertyFetch: Cannot access property $prop on null
}
