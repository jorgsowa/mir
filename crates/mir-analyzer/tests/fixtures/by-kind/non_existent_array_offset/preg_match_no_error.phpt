===description===
No error when accessing preg_match captures after success check (=== 1 or truthy)
===file===
<?php
// Exact reproducer: fresh variable, === 1 check
function nextName(string $filename): string
{
    if (preg_match('/^((.*) \()(-?\d+)(\)(\.[a-z]+)?)$/', $filename, $m) === 1) {
        return $m[1] . ((int) $m[3] + 1) . $m[4];
    }
    return $filename;
}

// Variable pre-declared as empty array, === 1 check
function test2(string $filename): string
{
    $m = [];
    if (preg_match('/^(foo)(bar)$/', $filename, $m) === 1) {
        return $m[1] . $m[2];
    }
    return $filename;
}

// Bare truthy check
function test3(string $filename): string
{
    if (preg_match('/^(foo)(bar)$/', $filename, $m)) {
        return $m[1] . $m[2];
    }
    return $filename;
}

// No condition — byref write-back gives array<int, string> from stub @param
function test4(string $filename): string
{
    preg_match('/^(foo)(bar)$/', $filename, $m);
    return $m[0];
}

// Reversed comparison: 1 === preg_match(...)
function test5(string $filename): string
{
    if (1 === preg_match('/^(foo)(bar)$/', $filename, $m)) {
        return $m[1];
    }
    return $filename;
}
===expect===
