===description===
Invalid extends final class and other annotation
===file===
<?php

/**
* @something-else-no-final annotation
*/
final class DoctrineA {}

class DoctrineB extends DoctrineA {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidExtendClass: Class DoctrineB cannot extend final class DoctrineA
