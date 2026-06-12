===description===
Capturing a variable in a closure use() clause (by value or by reference)
consumes the pending write — not UnusedVariable. Arrow function body reads
count too.
===file===
<?php
function takes(callable $cb): void { $cb(); }

function ensure_like(array $types): void {
    $allowed = $types;
    takes(function () use ($allowed) {
        var_dump($allowed);
    });
}

function unique_like(): void {
    $exists = [];
    takes(function () use (&$exists) {
        $exists[] = 1;
    });
}

function arrow_like(int $base): int {
    $offset = 10;
    $fn = fn (int $x): int => $x + $offset;
    return $fn($base);
}
===expect===
