===description===
new unknown class in method
===file===
<?php
class A {
    public function f(): void {
        new UnknownClass();
    }
}
===expect===
UndefinedClass@4:13: Class UnknownClass does not exist
