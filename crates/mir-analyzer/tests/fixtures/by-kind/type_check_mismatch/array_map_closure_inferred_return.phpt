===description===
array_map infers the result element type from a closure with no explicit return type
===config===
suppress=UnusedVariable,UnusedParam,MissingClosureReturnType
===file===
<?php
class Foo {}
class Bar {}
/** @param list<Foo> $items */
function test(array $items): void {
    $r = array_map(function (Foo $f) {
        return new Bar();
    }, $items);
    /** @mir-check $r is list<Bar> */
    $_ = $r;
}
===expect===
