===description===
array_count_values swaps the source's values into result keys (restricted
to int|string, since PHP 8 throws TypeError otherwise) and always produces
int<1, max> counts. A source whose values aren't entirely int|string (e.g.
array-of-arrays) can't succeed at runtime and isn't modeled — falls back
to the generic stub.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<string> $names
 * @param list<int> $maybe_empty_ids
 * @param list<array<string, int>> $rows
 */
function test(array $names, array $maybe_empty_ids, array $rows): void {
    $counts = array_count_values($names);
    /** @mir-check $counts is non-empty-array<string, int<1, max>> */
    $_ = $counts;

    $maybe_empty_counts = array_count_values($maybe_empty_ids);
    /** @mir-check $maybe_empty_counts is array<int, int<1, max>> */
    $_ = $maybe_empty_counts;

    $bad = array_count_values($rows);
    /** @mir-check $bad is array */
    $_ = $bad;
}
===expect===
