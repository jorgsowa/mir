===description===
Each loop pass joins only its own `continue` states, so a variable reshaped on the `continue` path still widens to a fixed point, in every loop kind and for `continue 2`.
===file===
<?php
/** @param list<bool> $xs */
function foreachContinueShape(array $xs): void {
    $s = 0;
    foreach ($xs as $x) {
        if ($x) { $s = [$s]; continue; }
        $s = ['k' => $s];
    }
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int>|int */
    echo 1;
}

function whileContinueShape(): void {
    $s = 0;
    while (rand(0, 1)) {
        if (rand(0, 1)) { $s = [$s]; continue; }
        $s = ['k' => $s];
    }
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int>|int */
    echo 1;
}

function forContinueShape(): void {
    $s = 0;
    for ($i = 0; $i < 10; $i++) {
        if (rand(0, 1)) { $s = [$s]; continue; }
        $s = ['k' => $s];
    }
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int> */
    echo 1;
}

function dowhileContinueShape(): void {
    $s = 0;
    do {
        if (rand(0, 1)) { $s = [$s]; continue; }
        $s = ['k' => $s];
    } while (rand(0, 1));
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int> */
    echo 1;
}

function infiniteLoopContinueShape(): void {
    $s = 0;
    while (true) {
        if (rand(0, 1)) { $s = [$s]; continue; }
        if (rand(0, 1)) { break; }
        $s = ['k' => $s];
    }
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int>|int */
    echo 1;
}

/** @param list<bool> $xs */
function outerContinueFromInnerLoop(array $xs): void {
    $s = 0;
    foreach ($xs as $x) {
        foreach ($xs as $y) {
            if ($x && $y) { $s = [$s]; continue 2; }
        }
        $s = ['k' => $s];
    }
    /** @mir-check $s is non-empty-array<string, non-empty-array<string, mixed>|non-empty-list|int>|non-empty-list<non-empty-array<string, mixed>|non-empty-list|int>|int */
    echo 1;
}

/** @param list<bool> $xs */
function counterAcrossContinue(array $xs): void {
    $n = 0;
    foreach ($xs as $x) {
        if ($x) { $n++; continue; }
        $n += 2;
    }
    /** @mir-check $n is int */
    echo 1;
}

/** @param list<string> $xs */
function firstIterationContinue(array $xs): string {
    $first = true;
    $out = '';
    foreach ($xs as $x) {
        if ($first) { $first = false; continue; }
        $out .= $x;
    }
    /** @mir-check $first is bool */
    echo 1;
    return $out;
}
