===description===
The interface in an `implements` clause lands on its declaration.
===cursor===
definition
===file===
<?php
interface Shape {}
final class Square implements Sha<CURSOR>pe {}
===expect===
test.php@2:0-2:18
