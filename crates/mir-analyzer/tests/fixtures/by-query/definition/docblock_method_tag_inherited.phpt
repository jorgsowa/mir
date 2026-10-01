===description===
A `@method` member reached through a subclass resolves to the parent's tag.
===cursor===
definition
===file===
<?php
/**
 * @method void ping()
 */
class Base {}
class Child extends Base {}
(new Child)->pi<CURSOR>ng();
===expect===
test.php@3:3-3:22
