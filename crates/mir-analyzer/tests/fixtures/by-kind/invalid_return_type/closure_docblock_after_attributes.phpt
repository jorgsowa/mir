===description===
A closure docblock between its attributes and function / fn applies, as in PHP; with docblocks on both sides the later one wins.
===file===
<?php
/** @return list<\Closure> */
function t(): array {
    $a = /** @param int $x */ #[\Deprecated] function ($x): string { return $x; };
//                                                                   ^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    $b = #[\Deprecated] /** @param int $x */ function ($x): string { return $x; };
//                                                                   ^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    $c = #[\Deprecated, \NoDiscard,] /** @param int $x */ static function ($x): string { return $x; };
//                                                                                       ^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    $d = #[\Deprecated] #[\Pure] /** @param int $x */ fn($x): string => $x;
//                                                                      ^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    $e = /** @param string $x */ #[\Deprecated] /** @param int $x */ fn($x): string => $x;
//                                                                                     ^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
    return [$a, $b, $c, $d, $e];
}
