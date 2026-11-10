===description===
`array_push()` with a spread argument may push nothing, so the array can still be empty afterwards; pushing a plain value makes it non-empty.
===file===
<?php
/** @return list<string> */
function wrap(string $line): array {
    return $line === '' ? [] : [$line];
}

/**
 * @param list<string> $lines
 * @return list<string>
 */
function wrapAll(array $lines): array {
    $result = [];
    foreach ($lines as $line) {
        $prefix = $result ? '> ' : '';
        array_push($result, ...wrap($prefix . $line));
    }
    return $result;
}

/** @param list<string> $extra */
function spreadOnly(array $extra): string {
    $all = [];
    array_push($all, ...$extra);
    return $all ? 'some' : 'none';
}

function plainValue(string $value): string {
    $all = [];
    array_push($all, $value);
    return $all ? 'some' : 'none';
//         ^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}
