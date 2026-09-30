===description===
new unknown class in method
===file===
<?php
class A {
    public function f(): void {
        new UnknownClass();
//          ^^^^^^^^^^^^ UndefinedClass: Class UnknownClass does not exist
    }
}
===expect===
