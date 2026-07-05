===description===
`class-string-map<T>` (Psalm's one-arg shorthand) reuses T as the value
type, matching `class-string-map<T, T>`, instead of silently degrading the
value type to `mixed`.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Foo {}

/** @return class-string-map<Foo> */
function makeMap() {
    return [];
}

$map = makeMap();
/** @mir-check $map is array<class-string, Foo> */
$_ = 1;
===expect===
