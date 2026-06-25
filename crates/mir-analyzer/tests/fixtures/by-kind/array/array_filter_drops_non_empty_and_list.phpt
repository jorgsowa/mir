===description===
array_filter strips the non-empty guarantee and the list re-indexing guarantee:
filtering can remove all entries (→ possibly-empty) and leaves gaps in integer keys
(→ plain array, not list). Key and value types from the source are preserved.
===config===
suppress=UnusedVariable,UnusedParam,MissingClosureReturnType
===file===
<?php
/**
 * @param non-empty-array<string, int> $assoc
 * @param non-empty-list<string> $words
 */
function test(array $assoc, array $words): void {
    // non-empty-array<string, int> → array<string, int> (no longer non-empty)
    $filtered_assoc = array_filter($assoc, fn(int $v) => $v > 0);
    /** @mir-check $filtered_assoc is array<string, int> */
    $_ = $filtered_assoc;

    // non-empty-list<string> → array<int, string> (no longer non-empty, no longer list)
    $filtered_words = array_filter($words, fn(string $w) => strlen($w) > 0);
    /** @mir-check $filtered_words is array<int, string> */
    $_ = $filtered_words;
}
===expect===
