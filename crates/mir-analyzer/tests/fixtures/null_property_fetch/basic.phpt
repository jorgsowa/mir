===source===
<?php
function test(): void {
    $x = null;
    echo $x->prop;
}
===expect===
NullPropertyFetch: $x->prop
