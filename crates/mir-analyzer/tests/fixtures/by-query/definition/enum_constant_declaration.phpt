===description===
An enum constant declaration name resolves to itself.
===cursor===
definition
===file===
<?php
enum Level {
    case Low;
    const DEF<CURSOR>AULT = self::Low;
}
===expect===
test.php@4:4-4:30
