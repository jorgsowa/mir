===description===
Find-references from a class constant declaration name lists the declaration and its uses.
===cursor===
references include_declaration
===file===
<?php
class Config { public const VER<CURSOR>SION = 1; }
echo Config::VERSION;
===expect===
test.php@2:28-2:35
test.php@3:13-3:20
