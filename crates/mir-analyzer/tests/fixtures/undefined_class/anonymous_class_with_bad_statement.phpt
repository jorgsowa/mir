===description===
anonymousClassWithBadStatement
===file===
<?php
$foo = new class {
    public function a() {
        new B();
    }
};
===expect===
UndefinedClass@4:12: Class B does not exist
