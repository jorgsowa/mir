===description===
An isset()-guarded map initialization followed by nested writes through a
dynamic outer key must retain each value's full keyed-array shape. array_values()
therefore returns the declared list shape rather than a union of partial wrappers.
===file===
<?php
/**
 * @param list<array{0: non-empty-string, 1: non-empty-string, 2: non-empty-string}> $entries
 * @return list<array{name: non-empty-string, tags: list<non-empty-string>, notes: list<non-empty-string>}>
 */
function collect(array $entries): array
{
    $byName = [];

    foreach ($entries as [$name, $tag, $note]) {
        if (!isset($byName[$name])) {
            $byName[$name] = ['name' => $name, 'tags' => [], 'notes' => []];
        }

        if ($tag !== '') {
            $byName[$name]['tags'][] = $tag;
        }

        if ($note !== '') {
            $byName[$name]['notes'][] = $note;
        }
    }

    return array_values($byName);
}
===expect===
