===description===
MixedArrayOffset fires even when the array access is guarded by null-coalesce — the offset is still mixed at the access point
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed $key */
$key = 'a';
$arr = ['a' => 1, 'b' => 2];
$val = $arr[$key] ?? 0;
===expect===
MixedArrayOffset@5:12-5:16: Mixed type used as array offset
