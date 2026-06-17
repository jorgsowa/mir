===description===
array_values returns a list preserving the source value type
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}
/** @param array<string, Foo> $arr */
function test(array $arr): void {
    $vals = array_values($arr);
    /** @mir-check $vals is list<Foo> */
    $_ = $vals;
}
===expect===
