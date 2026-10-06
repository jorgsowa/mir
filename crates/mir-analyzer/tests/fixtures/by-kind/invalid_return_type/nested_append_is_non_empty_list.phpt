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

/**
 * @param list<array> $rows
 * @return array<int, list<array{id: int}>>
 */
function appendedBareArraysFitShapeList(array $rows): array {
    $out = [];
    foreach ($rows as $row) {
        $out[$row['id']][] = $row;
    }
    return $out;
}

interface Event {}
interface Created extends Event {}

/**
 * @param list<Created> $events
 * @return array<int, non-empty-list<Event>>
 */
function appendedSubtypesFitBaseList(array $events): array {
    $out = [];
    foreach ($events as $event) {
        $out[1][] = $event;
    }
    return $out;
}

/**
 * @param list<object> $objects
 * @return array<int, non-empty-list<Event>>
 */
function appendedBareObjectsFitNamedList(array $objects): array {
    $out = [];
    foreach ($objects as $object) {
        $out[1][] = $object;
    }
    return $out;
}

/** @return array<string, non-empty-list<int>> */
function wrongElementTypeIsStillRejected(string $k): array {
    $out = [];
    $out[$k][] = 'text';
    return $out;
//  ^^^^^^^^^^^^ InvalidReturnType: Return type 'array<string, non-empty-list<"text">>' is not compatible with declared 'array<string, non-empty-list<int>>'
}
