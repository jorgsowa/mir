===description===
Sibling of array_reduce_infers_return_type: a genuinely wrong declared return type still errors.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param list<int> $ints
 * @return string
 */
function sumInts(array $ints): string {
    return array_reduce($ints, fn(int $c, int $x): int => $c + $x, 0);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'int' is not compatible with declared 'string'
}
===expect===
