===description===
`get_class($x) === Foo::class` must narrow $x to Foo, the same as the
`get_class($x) === 'Foo'` string-literal form already does — the
ClassConstAccess branch of the `===`/`!==` narrowing dispatch only ever
checked extract_var_name on the non-ClassConstAccess side, never
extract_get_class_arg, so a get_class(...) call on that side (rather than a
bare variable) fell through unnarrowed.
===file===
<?php

class Foo {}

function leftGetClass(object $x): void {
    if (get_class($x) === Foo::class) {
        /** @mir-check $x is Foo */
        echo "";
    }
}

function rightGetClass(object $x): void {
    if (Foo::class === get_class($x)) {
        /** @mir-check $x is Foo */
        echo "";
    }
}
===expect===
