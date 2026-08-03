===description===
P26: preg_split()'s non-empty-list<string> return, once merged with an
`=== false` guard's single-element array-literal fallback (`[$string]`),
must widen the foreach key type to plain `int` — not collapse to the
fallback shape's literal `int(0)` key, which made `$index % 2 === 1`
look like an always-false comparison.
===config===
suppress=UnusedParam
===file===
<?php

function odd_segments(string $pattern, string $string): array {
    $segments = preg_split($pattern, $string, -1, PREG_SPLIT_DELIM_CAPTURE);
    if ($segments === false) {
        $segments = [$string];
    }
    $result = [];
    foreach ($segments as $index => $segment) {
        if ($index % 2 === 1) {
            $result[] = $segment;
        }
    }
    return $result;
}
===expect===
