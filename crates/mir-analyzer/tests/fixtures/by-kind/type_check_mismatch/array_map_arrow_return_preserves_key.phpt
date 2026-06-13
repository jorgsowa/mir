===description===
array_map with an arrow fn refines the result to array<sourceKey, callbackReturn>
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Foo {}
/** @param array<string, Foo> $arr */
function test(array $arr): void {
    $r = array_map(fn(Foo $f): int => 1, $arr);
    /** @mir-check $r is array<string, int> */
    $_ = $r;
}
===expect===
