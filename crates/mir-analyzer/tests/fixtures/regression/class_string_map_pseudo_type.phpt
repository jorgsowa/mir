===description===
`class-string-map<T, V>` is approximated as `array<class-string, V>` instead
of being misparsed as a bogus named class.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Foo {}

/** @return class-string-map<Foo, Foo> */
function makeMap() {
    return [];
}

$map = makeMap();
/** @mir-check $map is array<class-string, Foo> */

===expect===
