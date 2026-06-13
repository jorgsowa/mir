===description===
Anonymous class with bad statement
===config===
suppress=UnusedVariable
===file===
<?php
$foo = new class {
    public function a() {
        new B();
    }
};
===expect===
UndefinedClass@4:13-4:14: Class B does not exist
