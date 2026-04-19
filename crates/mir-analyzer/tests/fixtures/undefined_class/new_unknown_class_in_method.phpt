===source===
<?php
class A {
    public function f(): void {
        new UnknownClass();
    }
}
===expect===
UndefinedClass: UnknownClass
