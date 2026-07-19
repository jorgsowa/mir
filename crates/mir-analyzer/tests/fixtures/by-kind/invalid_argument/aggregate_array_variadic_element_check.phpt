===description===
A variadic param documented as an aggregate array (`array<int, V>
...$args`), not just the bare-element or list<V> spellings, must check
each argument against V — check_args only unwrapped TList/TNonEmptyList,
so every argument was wrongly compared against the whole array<K,V> type.
===config===
suppress=MissingThrowsDocblock,UnusedParam
===file===
<?php
/** @param array<int, int> ...$nums */
function sumAll(...$nums): int {
    return array_sum($nums);
}

sumAll(1, 2, 3);
sumAll(1, "bad");
===expect===
InvalidArgument@8:10-8:15: Argument $nums of sumAll() expects 'int', got '"bad"'
