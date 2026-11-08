===description===
`['Ns\Class', 'method']` is a callable when the literal names a class.
===file===
<?php
namespace App;
class SortArray {
    public static function byW(array $a, array $b): int { return $a <=> $b; }
}
function f(array $arr): array {
    uasort($arr, ['App\SortArray', 'byW']);
    uasort($arr, ['\App\SortArray', 'byW']);
    return $arr;
}
