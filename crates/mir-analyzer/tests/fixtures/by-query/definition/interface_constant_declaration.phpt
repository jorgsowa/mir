===description===
An interface constant declaration name resolves to itself.
===cursor===
definition
===file===
<?php
interface HasLimit {
    const LIM<CURSOR>IT = 5;
}
===expect===
test.php@3:4-3:20
