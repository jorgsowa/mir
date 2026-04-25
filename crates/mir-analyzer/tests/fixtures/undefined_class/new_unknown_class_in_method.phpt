===file===
<?php
class A {
    public function f(): void {
        new UnknownClass();
    }
}
===expect===
UndefinedClass: Class UnknownClass does not exist
