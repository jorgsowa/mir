===description===
The class token of a class-constant fetch resolves to the class, like the class token of a static call.
===ignore===
===cursor===
symbol
===file===
<?php
final class Status { public const ACTIVE = 1; }
echo Sta<CURSOR>tus::ACTIVE;
===expect===
kind: class Status
type: class-string
