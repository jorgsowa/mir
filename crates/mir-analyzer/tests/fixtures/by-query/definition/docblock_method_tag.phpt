===description===
A `@method` member resolves to its tag, not the class declaration.
===cursor===
definition
===file===
<?php
/**
 * @method string magic()
 * @method static self make()
 */
class Magic {}
function run(Magic $m): void {
    $m->mag<CURSOR>ic();
}
===expect===
test.php@3:3-3:25
