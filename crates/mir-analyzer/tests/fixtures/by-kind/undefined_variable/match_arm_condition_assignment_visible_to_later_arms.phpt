===description===
Match arm conditions run in order, so a variable assigned in an earlier arm's condition is defined in later arms; narrowing inside a condition stays in its arm.
===file===
<?php
/** @param array<string, string> $map */
function pick(array $map, string $id): string {
    return match (true) {
        !$method = $map[$id] ?? null => 'none',
        $method === 'a' => 'A',
        default => 'other',
    };
}

function precision(string $s): string {
    return match (true) {
        !$length = strlen($s) => 'c',
        $length < 4 => 'short',
        default => 'long',
    };
}

function unset_later(int $n): mixed {
    return match (true) {
        $n > 0 => $n,
        default => $later,
//                 ^^^^^^ UndefinedVariable: Variable $later is not defined
    };
}

/** @param string|list<string> $c */
function narrowing_stays_in_its_arm(string|array $c): string {
    return match (true) {
        \is_array($c) && $c !== [] => 'array',
        default => \is_string($c) ? $c : 'empty',
    };
}
