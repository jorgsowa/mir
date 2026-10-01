===description===
A class constant declaration name resolves to itself.
===cursor===
definition
===file===
<?php
class Config {
    public const VER<CURSOR>SION = 1;
}
echo Config::VERSION;
===expect===
test.php@3:4-3:29
