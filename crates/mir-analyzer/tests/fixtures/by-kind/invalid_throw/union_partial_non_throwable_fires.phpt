===description===
Union type where one part is not Throwable fires InvalidThrow
===file===
<?php
/** @var \RuntimeException|string $e */
$e = new \RuntimeException();
throw $e;
//<^^^^^^^^^ InvalidThrow: Thrown type 'RuntimeException|string' does not extend Throwable
===expect===
