===description===
MixedArrayOffset does not fire inside an is_string() guard when the variable was declared @var mixed — is_string() narrows mixed to a concrete string type.
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
