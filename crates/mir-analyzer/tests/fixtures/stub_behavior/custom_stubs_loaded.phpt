===file===
<?php
// stdClass is defined in custom stubs — instantiation must not produce UndefinedClass
$obj = new stdClass();
$obj->name = 'test';
===expect===
