===description===
Wrong case class name in static call is reported.
===file===
<?php
class MyClass {
    public static function hello(): void {}
}
myclass::hello();
//<^^^^^^^ WrongCaseClass: Class name 'myclass' has incorrect casing; use 'MyClass'
===expect===
