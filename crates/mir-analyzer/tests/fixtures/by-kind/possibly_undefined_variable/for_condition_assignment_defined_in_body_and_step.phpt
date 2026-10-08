===description===
A `for` condition runs before the body and before the step, so variables it
assigns are defined in both, and after the loop (the condition runs at least once).
===file===
<?php
function scan(string $s, int $n): int {
    $last = 0;
    for ($t = 0; ($p = strlen(substr($s, $t))) < $n; $t = $p + 1) {
        /** @mir-check $p is int<0, max> */
        $last = $p;
    }
    return $p + $last;
}

/** @param array<string, int> $a */
function drain(array $a): void {
    for ($n = 0; $k = key($a);) {
        echo $k, $n;
        next($a);
    }
}

function everyConditionRuns(int $n): void {
    for ($i = 0; $a = $i, $b = $i + 1, $i < $n; $i++) {
        echo $a + $b;
    }
    echo $a + $b;
}
