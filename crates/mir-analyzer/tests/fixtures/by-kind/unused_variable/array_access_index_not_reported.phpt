===description===
array access index not reported
===config===
<mir>
  <issueHandlers>
    <MixedArrayOffset errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $arr): mixed {
    $keys = array_keys($arr);
    return $arr[$keys[0]];
}
