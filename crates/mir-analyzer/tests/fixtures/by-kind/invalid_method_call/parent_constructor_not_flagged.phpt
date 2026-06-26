===description===
parent::__construct() is a static call and is not flagged
===file===
<?php
class Base {
    public function __construct() {}
}
class Child extends Base {
    public function __construct() { parent::__construct(); }
}
===expect===
