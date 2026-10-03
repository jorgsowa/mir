===description===
A variadic param documented as an aggregate array (`array<int, V>
...$args`), not just the bare-element or list<V> spellings, must check
each argument against V — check_args only unwrapped TList/TNonEmptyList,
so every argument was wrongly compared against the whole array<K,V> type.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<int, int> ...$nums */
function sumAll(...$nums): int {
    return array_sum($nums);
}

sumAll(1, 2, 3);
sumAll(1, "bad");
//        ^^^^^ InvalidArgument: Argument $nums of sumAll() expects 'int', got '"bad"'
===expect===
