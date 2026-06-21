===description===
Invalid extends annotated final class
===file===
<?php

/**
* @final
*/
class DoctrineA {}

class DoctrineB extends DoctrineA {}

===expect===
InvalidExtendClass@8:0-8:36: Class DoctrineB cannot extend final class DoctrineA
