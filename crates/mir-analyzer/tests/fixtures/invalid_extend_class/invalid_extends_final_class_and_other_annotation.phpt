===description===
Invalid extends final class and other annotation
===file===
<?php

/**
* @something-else-no-final annotation
*/
final class DoctrineA {}

class DoctrineB extends DoctrineA {}'

===expect===
InvalidExtendClass
===ignore===
TODO
