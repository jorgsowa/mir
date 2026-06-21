===description===
@final is a soft docblock convention — the PHP `final` keyword enforces non-extensibility,
but @final alone must not emit InvalidExtendClass (it is an IDE hint, not a PHP rule).
===file===
<?php

/**
* @final
*/
class DoctrineA {}

class DoctrineB extends DoctrineA {}

===expect===
