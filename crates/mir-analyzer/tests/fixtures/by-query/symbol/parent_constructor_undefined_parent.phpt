===description===
`parent::__construct()` resolves to the method on the literal extends class even when it is undeclared.
===cursor===
symbol
===file===
<?php
class Child extends Missing {
    public function __construct() {
        parent::__con<CURSOR>struct();
    }
}
===expect===
kind: static call Missing::__construct
type: mixed
