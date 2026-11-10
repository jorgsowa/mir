===description===
The key and value of a template-typed array are unknown until the template is bound, so `is_string()`/`is_int()` on them is undecided.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template TValue of array<array|scalar>|scalar
 * @param TValue $value
 * @return list<string>
 */
function stringKeys(mixed $value): array {
    $names = [];
    if (is_array($value)) {
        foreach (array_keys($value) as $key) {
            if (is_string($key)) {
                $names[] = $key;
            }
            $names[] = is_int($key) ? '#' . $key : $key;
        }
    }
    return $names;
}

/**
 * @template TItems of array<int|string>
 * @param TItems $items
 */
function sumInts(array $items): int {
    $total = 0;
    foreach ($items as $item) {
        $total += is_int($item) ? $item : strlen($item);
    }
    return $total;
}
