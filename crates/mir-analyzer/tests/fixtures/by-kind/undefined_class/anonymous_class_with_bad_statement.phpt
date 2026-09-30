===description===
Anonymous class with bad statement
===config===
suppress=UnusedVariable
===file===
<?php
$foo = new class {
    public function a() {
        new B();
//          ^ UndefinedClass: Class B does not exist
    }
};
===expect===
