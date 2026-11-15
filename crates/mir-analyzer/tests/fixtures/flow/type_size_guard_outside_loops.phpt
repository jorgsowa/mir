===description===
Types built without a loop stay bounded: array nesting past 8 levels collapses, while shallow shapes, wide object unions and literal strings stay exact and code reading them reports nothing.
===file===
<?php
function nestedArrayLiteral(): void {
    $a = [[[[[[[[[[[1]]]]]]]]]]];
    /** @mir-check $a is array{0: array{0: array{0: array{0: array{0: array{0: array{0: non-empty-list}}}}}}} */
    var_dump($a);
}

function sequentialWrapping(): void {
    $r = 0;
    $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r];
    $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r]; $r = ['v' => $r];
    /** @mir-check $r is array{'v': array{'v': array{'v': array{'v': array{'v': array{'v': array{'v': non-empty-array<string, mixed>}}}}}}} */
    var_dump($r);
}

function shallowNestingStaysExact(): void {
    $config = ['db' => ['primary' => ['host' => 'localhost', 'port' => 5432]]];
    /** @mir-check $config is array{'db': array{'primary': array{'host': "localhost", 'port': 5432}}} */
    var_dump($config);
}

function deepConditionalWrapping(int $n): int {
    $v = 0;
    if ($n === 1) { $v = [$v]; } if ($n === 2) { $v = [$v]; } if ($n === 3) { $v = [$v]; }
    if ($n === 4) { $v = [$v]; } if ($n === 5) { $v = [$v]; } if ($n === 6) { $v = [$v]; }
    if ($n === 7) { $v = [$v]; } if ($n === 8) { $v = [$v]; } if ($n === 9) { $v = [$v]; }
    if ($n === 10) { $v = [$v]; } if ($n === 11) { $v = [$v]; } if ($n === 12) { $v = [$v]; }
    return is_int($v) ? $v : count($v);
}

class O1 {} class O2 {} class O3 {} class O4 {} class O5 {} class O6 {} class O7 {} class O8 {}
class O9 {} class O10 {} class O11 {} class O12 {} class O13 {} class O14 {} class O15 {} class O16 {}
class O17 {} class O18 {} class O19 {} class O20 {} class O21 {} class O22 {} class O23 {} class O24 {}
class O25 {} class O26 {} class O27 {} class O28 {} class O29 {} class O30 {} class O31 {} class O32 {}

function wideObjectUnionKeepsClasses(int $n): void {
    $o = match ($n) {
        1 => new O1, 2 => new O2, 3 => new O3, 4 => new O4, 5 => new O5, 6 => new O6, 7 => new O7, 8 => new O8,
        9 => new O9, 10 => new O10, 11 => new O11, 12 => new O12, 13 => new O13, 14 => new O14, 15 => new O15, 16 => new O16,
        17 => new O17, 18 => new O18, 19 => new O19, 20 => new O20, 21 => new O21, 22 => new O22, 23 => new O23, 24 => new O24,
        25 => new O25, 26 => new O26, 27 => new O27, 28 => new O28, 29 => new O29, 30 => new O30, 31 => new O31, default => new O32,
    };
    /** @mir-check $o is O1|O2|O3|O4|O5|O6|O7|O8|O9|O10|O11|O12|O13|O14|O15|O16|O17|O18|O19|O20|O21|O22|O23|O24|O25|O26|O27|O28|O29|O30|O31|O32 */
    echo 1;
    if ($o instanceof O32) { echo "last"; }
}

function wideLiteralStringsStayExact(int $n): string {
    $s = 's0';
    if ($n === 1) { $s = 's1'; } if ($n === 2) { $s = 's2'; } if ($n === 3) { $s = 's3'; } if ($n === 4) { $s = 's4'; }
    if ($n === 5) { $s = 's5'; } if ($n === 6) { $s = 's6'; } if ($n === 7) { $s = 's7'; } if ($n === 8) { $s = 's8'; }
    if ($n === 9) { $s = 's9'; } if ($n === 10) { $s = 's10'; } if ($n === 11) { $s = 's11'; } if ($n === 12) { $s = 's12'; }
    if ($n === 13) { $s = 's13'; } if ($n === 14) { $s = 's14'; } if ($n === 15) { $s = 's15'; } if ($n === 16) { $s = 's16'; }
    if ($n === 17) { $s = 's17'; } if ($n === 18) { $s = 's18'; } if ($n === 19) { $s = 's19'; } if ($n === 20) { $s = 's20'; }
    if ($n === 21) { $s = 's21'; } if ($n === 22) { $s = 's22'; } if ($n === 23) { $s = 's23'; } if ($n === 24) { $s = 's24'; }
    if ($n === 25) { $s = 's25'; } if ($n === 26) { $s = 's26'; } if ($n === 27) { $s = 's27'; } if ($n === 28) { $s = 's28'; }
    if ($n === 29) { $s = 's29'; } if ($n === 30) { $s = 's30'; } if ($n === 31) { $s = 's31'; }
    if ($s === 's31') { return 'last'; }
    return $s;
}

function arrayMergeChain(): int {
    $r = ['depth' => 0];
    $r = array_merge($r, ['next' => $r]); $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]); $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]); $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]); $r = array_merge($r, ['next' => $r]);
    $r = array_merge($r, ['next' => $r]); $r = array_merge($r, ['next' => $r]);
    return count($r);
}
