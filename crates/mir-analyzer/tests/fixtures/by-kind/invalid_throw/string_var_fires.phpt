===description===
Throwing a string variable fires InvalidThrow
===file===
<?php
/** @var string $e */
$e = 'error message';
throw $e;
//<^^^^^^^^^ InvalidThrow: Thrown type 'string' does not extend Throwable
