===description===
Constructor
===ignore===
TODO
===file===
<?php
/**
 * @consistent-constructor
 */
class C {
    public function __construct() {}
}

class C2 extends C {
    #[Override]
    public function __construct() {}
}

===expect===
