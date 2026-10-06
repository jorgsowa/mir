===description===
`+` with an array on one side and `mixed` on the other is the array-union
operation when the mixed value is an array, so it is not definitely invalid.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function mixedLeft(mixed $left): array {
    return $left + [1];
}

function mixedRight(mixed $right): array {
    return [1] + $right;
}

/** @param list<int>|array<string, int> $shape */
function unionArray(mixed $left, array $shape): array {
    return $left + $shape;
}
