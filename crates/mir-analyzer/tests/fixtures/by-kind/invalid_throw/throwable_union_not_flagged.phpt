===description===
Union of two Throwable types does not fire InvalidThrow
===file===
<?php
/** @var \RuntimeException|\LogicException $e */
$e = new \RuntimeException();
throw $e;
===expect===
