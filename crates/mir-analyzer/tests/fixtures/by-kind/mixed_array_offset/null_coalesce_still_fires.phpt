===description===
MixedArrayOffset fires even when the array access is guarded by null-coalesce — the offset is still mixed at the access point
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var mixed $key */
$key = 'a';
$arr = ['a' => 1, 'b' => 2];
$val = $arr[$key] ?? 0;
//          ^^^^ MixedArrayOffset: Mixed type used as array offset
===expect===
