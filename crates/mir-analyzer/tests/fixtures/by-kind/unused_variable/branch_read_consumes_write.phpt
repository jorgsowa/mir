===description===
A write read on ANY path is used: ternary arms, match arms, and the
loop-never-ran path after a foreach that overwrites the variable. Genuinely
dead writes are still reported.
===config===
suppress=MissingReturnType,MixedAssignment
===file===
<?php
function ternary_read(?int $number): array {
    $result = [1, 2, 3];
    return is_null($number) ? $result : array_slice($result, 0, $number);
}

function match_read(int $k): int {
    $fallback = 10;
    return match ($k) {
        1 => 1,
        default => $fallback,
    };
}

function last_like(iterable $items) {
    $needle = $placeholder = new stdClass;
    foreach ($items as $value) {
        $needle = $value;
    }
    return $needle === $placeholder ? null : $needle;
}

function still_dead(): int {
    $x = 1;
    $x = 2;
    return $x;
}
===expect===
UnusedVariable@24:5-24:7: Variable $x is never read
