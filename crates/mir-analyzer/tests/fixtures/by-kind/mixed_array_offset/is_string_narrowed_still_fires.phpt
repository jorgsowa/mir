===description===
MixedArrayOffset still fires inside an is_string() guard when the variable was declared @var mixed — TMixed is not narrowed away by is_string()
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed $key */
$key = 'hello';
$arr = ['hello' => 1, 'world' => 2];
if (is_string($key)) {
    $val = $arr[$key];
}
===expect===
MixedArrayOffset@6:16-6:20: Mixed type used as array offset
