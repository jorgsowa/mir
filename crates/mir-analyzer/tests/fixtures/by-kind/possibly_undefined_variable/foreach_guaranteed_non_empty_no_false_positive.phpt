===description===
foreach over a provably non-empty array is guaranteed to execute at least
once, so a variable first assigned in the body is definitely defined
afterward — no PossiblyUndefinedVariable, unlike a possibly-empty source
(see foreach_body_error.phpt, which correctly still flags it).
===config===
suppress=MixedAssignment,MixedReturnStatement
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

/** @param array{a: int}|array<int, int> $items */
function mixed_shape_union_not_guaranteed(array $items): int {
    foreach ($items as $item) {
        $last = $item;
    }
    return $last;
}
===expect===
PossiblyUndefinedVariable@22:11-22:16: Variable $last might not be defined
