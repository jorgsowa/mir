===description===
foreach over a provably non-empty array is guaranteed to execute at least
once, so a variable first assigned in the body is definitely defined
afterward — no PossiblyUndefinedVariable, unlike a possibly-empty source
(see foreach_body_error.phpt, which correctly still flags it).
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function literal_array_guaranteed(): int {
    foreach ([1, 2, 3] as $item) {
        $last = $item;
    }
    return $last;
}

/** @param non-empty-array<int> $items */
function non_empty_array_param_guaranteed(array $items): int {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
}

/** @param array{id: int, name?: string} $items */
function required_shape_guaranteed(array $items): int {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
}

/** @param array{a?: int} $items */
function all_optional_shape_not_guaranteed(array $items): int {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
//         ^^^^^ PossiblyUndefinedVariable: Variable $last might not be defined
}

/** @param array{a: int}|array<int, int> $items */
function mixed_shape_union_not_guaranteed(array $items): int {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
//         ^^^^^ PossiblyUndefinedVariable: Variable $last might not be defined
}
