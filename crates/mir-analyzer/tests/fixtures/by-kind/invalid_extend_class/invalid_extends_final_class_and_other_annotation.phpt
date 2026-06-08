===description===
Invalid extends final class and other annotation
===file===
<?php

/**
* @something-else-no-final annotation
*/
final class DoctrineA {}

class DoctrineB extends DoctrineA {}

===expect===
InvalidExtendClass@8:0-8:36: Class DoctrineB cannot extend final class DoctrineA
