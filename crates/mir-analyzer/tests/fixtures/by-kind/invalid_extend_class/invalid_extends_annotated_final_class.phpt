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
InvalidExtendClass@8:0-8:37: Class DoctrineB cannot extend final class DoctrineA
ParseError@8:36-8:37: Parse error: unterminated string literal
ParseError@8:37-8:37: Parse error: expected ';' after expression
