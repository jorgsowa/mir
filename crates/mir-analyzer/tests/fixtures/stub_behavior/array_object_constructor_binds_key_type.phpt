===description===
`new ArrayObject($array)` binds both the key and the value type from the
array, so the object satisfies a parameter typed with those keys.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param ArrayAccess<string, int>&Countable $obj */
function acceptsMap(ArrayAccess&Countable $obj): void {}

$map = new ArrayObject(['a' => 1]);
/** @mir-check $map is ArrayObject<string, int> */
acceptsMap($map);

$list = new ArrayObject([1, 2]);
acceptsMap($list);
//         ^^^^^ InvalidArgument: Argument $obj of acceptsMap() expects 'ArrayAccess<string, int>&Countable', got 'ArrayObject<int, int>'
