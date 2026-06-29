===description===
Union type where one part is not Throwable fires InvalidThrow
===file===
<?php
/** @var \RuntimeException|string $e */
$e = new \RuntimeException();
throw $e;
===expect===
InvalidThrow@4:0-4:9: Thrown type 'RuntimeException|string' does not extend Throwable
