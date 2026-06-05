===description===
Wrong case trait name in use statement inside a class is reported.
===file===
<?php
trait Greetable {
    public function greet(): void {}
}
class Person {
    use greetable;
}
===expect===
WrongCaseClass@5:0-5:14: Class name 'greetable' has incorrect casing; use 'Greetable'
