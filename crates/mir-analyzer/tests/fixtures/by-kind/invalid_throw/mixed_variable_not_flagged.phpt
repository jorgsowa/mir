===description===
Throwing a mixed variable does not fire InvalidThrow — cannot statically determine type
===file===
<?php
/** @var mixed $e */
$e = new \RuntimeException();
throw $e;
===expect===
