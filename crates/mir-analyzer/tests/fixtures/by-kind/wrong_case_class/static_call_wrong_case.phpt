===description===
Wrong case class name in static call is reported.
===file===
<?php
class MyClass {
    public static function hello(): void {}
}
myclass::hello();
===expect===
WrongCaseClass@5:1-5:8: Class name 'myclass' has incorrect casing; use 'MyClass'
