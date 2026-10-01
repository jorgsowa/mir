===description===
`parent::__construct()` resolves to the method on the literal extends class even when it is undeclared.
===cursor===
references
===file===
<?php
class Child extends Missing {
    public function __construct() {
        parent::__con<CURSOR>struct();
    }
}
===expect===
test.php@4:16-4:27
