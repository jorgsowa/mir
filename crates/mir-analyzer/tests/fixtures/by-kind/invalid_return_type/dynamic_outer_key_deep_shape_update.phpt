===description===
Known multi-level keys under a dynamic map key remain attached to the existing
shape. A regular nested assignment and a list push both preserve sibling fields.
===file===
<?php
/**
 * @param list<array{0: string, 1: string}> $rows
 * @return array<string, array{state: array{count: int, labels: list<string>}, enabled: bool}>
 */
function index(array $rows): array
{
    $indexed = [];

    foreach ($rows as [$id, $label]) {
        if (!isset($indexed[$id])) {
            $indexed[$id] = [
                'state' => ['count' => 0, 'labels' => []],
                'enabled' => true,
            ];
        }

        $indexed[$id]['state']['count'] = 1;
        $indexed[$id]['state']['labels'][] = $label;
    }

    return $indexed;
}
===expect===
