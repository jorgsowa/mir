===description===
Ternary conditions that can go either way are not reported: unions, nullables, mixed, loop-carried flags and the short `?:` form.
===file===
<?php
final class Token {}

function union(int|string $id): string {
    return is_string($id) ? 'string' : 'int';
}

function nullable(?Token $token): string {
    return $token === null ? 'none' : 'token';
}

function nullableInstanceof(?Token $token): string {
    return $token instanceof Token ? 'token' : 'none';
}

function fromMixed(mixed $value): string {
    return $value instanceof Token ? 'token' : 'other';
}

function shortForm(string $name): string {
    return $name ?: 'anonymous';
}

/** @param list<string> $parts */
function loopFlag(array $parts): string {
    $first = true;
    $out = '';
    foreach ($parts as $part) {
        $out .= $first ? $part : ', ' . $part;
        $first = false;
    }
    return $out;
}

function counter(int $limit): int {
    $total = 0;
    for ($i = 0; $i < $limit; $i++) {
        $total += $i === 0 ? 1 : 2;
    }
    return $total;
}
