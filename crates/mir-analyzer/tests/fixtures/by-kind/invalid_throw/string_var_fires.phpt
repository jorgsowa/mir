===description===
Throwing a string variable fires InvalidThrow
===file===
<?php
/** @var string $e */
$e = 'error message';
throw $e;
===expect===
InvalidThrow@4:0-4:9: Thrown type 'string' does not extend Throwable
