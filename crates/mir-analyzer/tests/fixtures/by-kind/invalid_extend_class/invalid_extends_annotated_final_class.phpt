===description===
Invalid extends annotated final class
===file===
<?php

/**
* @final
*/
class DoctrineA {}

class DoctrineB extends DoctrineA {}'

===expect===
InvalidExtendClass
===ignore===
TODO
