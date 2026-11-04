===description===
array_merge and array_replace keep the key type of arrays with mixed values
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array<string, mixed> $a
 * @param array<string, mixed> $b
 * @param array<int, string> $ints
 */
function f(array $a, array $b, array $ints): void {
    $merged = array_merge($a, $b);
    /** @mir-check $merged is array<string, mixed> */
    $replaced = array_replace($a, $b);
    /** @mir-check $replaced is array<string, mixed> */
    $replacedInts = array_replace($ints, [5 => 'x']);
    /** @mir-check $replacedInts is non-empty-array<int, string> */
    echo 1;
}
