===description===
Go-to-definition on the class token of a class-constant fetch lands on the class.
===ignore===
===cursor===
definition
===file===
<?php
final class Status { public const ACTIVE = 1; }
echo Sta<CURSOR>tus::ACTIVE;
===expect===
test.php@2:6-2:47
