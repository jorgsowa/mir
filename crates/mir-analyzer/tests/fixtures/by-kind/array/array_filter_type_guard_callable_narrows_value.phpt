===description===
`array_filter()` with a string or first-class callable naming a type guard narrows the values like the equivalent closure.
===file===
<?php
/** @param array<string, int|string> $m */
function stringCallable(array $m): void {
    $r = array_filter($m, 'is_string');
    /** @mir-check $r is array<string, string> */
    echo count($r);
}

/** @param array<string, int|string> $m */
function firstClassCallable(array $m): void {
    $r = array_filter($m, is_string(...));
    /** @mir-check $r is array<string, string> */
    echo count($r);
}

/** @param array<string, int|string|null> $m */
function closureForm(array $m): void {
    $r = array_filter($m, fn($v) => is_string($v));
    /** @mir-check $r is array<string, string> */
    echo count($r);
}

/** @param array<string, int|string> $m */
function keepsTypeForNonGuard(array $m): void {
    $r = array_filter($m, 'strlen');
    /** @mir-check $r is array<string, int|string> */
    echo count($r);
}

/** @param array<string, int|string> $m */
function keepsTypeForUseKeyMode(array $m): void {
    $r = array_filter($m, 'is_string', ARRAY_FILTER_USE_KEY);
    /** @mir-check $r is array<string, int|string> */
    echo count($r);
}
===expect===
