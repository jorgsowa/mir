===description===
Go-to-definition on a class declaration name resolves to the class itself.
===cursor===
definition
===file===
<?php
class Fo<CURSOR>o {}
===expect===
test.php@2:0-2:12
