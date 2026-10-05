===description===
Appending to a nested list makes that list non-empty
===file===
<?php
/**
 * @param list<int> $rows
 * @return array<int, non-empty-list<string>>
 */
function groupedByRow(array $rows): array {
    $out = [];
    foreach ($rows as $r) {
        if ($r < 1) {
            continue;
        }
        $out[$r][] = 'x';
    }
    return $out;
}

/** @return array<string, non-empty-list<int>> */
function afterTwoAppends(string $k): array {
    $out = [];
    $out[$k][] = 1;
    $out[$k][] = 2;
    return $out;
}

/** @return array<string, non-empty-list<int>> */
function threeLevelsDeep(string $a, string $b): array {
    $out = [];
    $out[$a][$b][] = 1;
    return $out[$a];
}

/**
 * @param array<string, list<int>> $existing
 * @return array<string, list<int>>
 */
function existingListsStayLists(array $existing, string $k): array {
    $existing[$k][] = 1;
    return $existing;
}

/** @return array<string, non-empty-list<int>> */
function wrongElementTypeIsStillRejected(string $k): array {
    $out = [];
    $out[$k][] = 'text';
    return $out;
}
===expect===
InvalidReturnType@45:4-45:16: Return type 'array<string, non-empty-list<"text">>' is not compatible with declared 'array<string, non-empty-list<int>>'
