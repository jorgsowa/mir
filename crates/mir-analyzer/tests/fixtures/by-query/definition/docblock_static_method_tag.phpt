===description===
A static `@method` member resolves to its tag.
===cursor===
definition
===file===
<?php
/**
 * @method string magic()
 * @method static self make()
 */
class Magic {}
Magic::ma<CURSOR>ke();
===expect===
test.php@4:3-4:29
