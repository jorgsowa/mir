===description===
array_map on a non-empty array returns a non-empty array; array_map on a possibly-empty array returns a possibly-empty array
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param non-empty-array<int> $a
 * @param array<int> $b
 */
function test(array $a, array $b): void {
    $mapped_a = array_map(fn($x) => $x * 2, $a);
    /** @mir-check $mapped_a is non-empty-array<int|string, int> */
    $_ = $mapped_a;

    $mapped_b = array_map(fn($x) => $x * 2, $b);
    /** @mir-check $mapped_b is array<int|string, int> */
    $_ = $mapped_b;
}
