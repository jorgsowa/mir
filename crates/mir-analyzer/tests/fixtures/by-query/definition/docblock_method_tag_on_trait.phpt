===description===
A `@method` member declared on a trait resolves to its tag.
===cursor===
definition
===file===
<?php
/**
 * @method void ping()
 */
trait Pings {}
class Host { use Pings; }
(new Host)->pi<CURSOR>ng();
===expect===
test.php@3:3-3:22
