===description===
array_filter preserves the source array key and value types
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}
/** @param array<string, Foo> $arr */
function test(array $arr): void {
    $r = array_filter($arr, fn(Foo $f): bool => true);
    /** @mir-check $r is array<string, Foo> */
    $_ = $r;
}
===expect===
