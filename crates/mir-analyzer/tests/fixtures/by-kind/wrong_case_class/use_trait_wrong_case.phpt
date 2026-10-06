===description===
Wrong case trait name in use statement inside a class is reported.
===file===
<?php
trait Greetable {
    public function greet(): void {}
}
class Person {
//<^^^^^^^^^^^^^^ WrongCaseClass: Class name 'greetable' has incorrect casing; use 'Greetable'
    use greetable;
}
