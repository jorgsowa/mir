===description===
A trait property declaration name resolves to itself.
===cursor===
definition
===file===
<?php
trait Counts {
    protected int $co<CURSOR>unt = 0;
}
===expect===
test.php@3:4-3:28
