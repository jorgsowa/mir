===description===
Loosening the trailing comparator slot to `mixed` must not stop real
argument-type errors on the preceding array parameters.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$cmp = function ($x, $y) { return $x <=> $y; };
array_udiff('not an array', ['b' => 2], $cmp);
//          ^^^^^^^^^^^^^^ InvalidArgument: Argument $array of array_udiff() expects 'array', got '"not an array"'
