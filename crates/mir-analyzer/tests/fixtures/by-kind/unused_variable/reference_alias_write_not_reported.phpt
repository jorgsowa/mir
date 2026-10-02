===description===
Writes through a variable bound with `=&` mutate the referent, so the alias is not reported as unused.
===config===
suppress=MixedAssignment,MixedArrayAccess
===file===
<?php
/** @param list<array{g: int, id: int}> $rows */
function group(array $rows): array {
    $groups = [];
    foreach ($rows as $r) {
        $bucket = &$groups[$r['g']]['items'];
        $bucket[] = $r['id'];
    }
    return $groups;
}

function alias(int $n): int {
    $total = 0;
    $ref = &$total;
    $ref = $n;
    return $total;
}
===expect===
