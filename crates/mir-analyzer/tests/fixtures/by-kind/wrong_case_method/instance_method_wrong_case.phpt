===description===
Calling an instance method with wrong casing is reported.
===file===
<?php
class Greeter {
    public function sayHello(): void {}
}
$g = new Greeter();
$g->SAYhello();
===expect===
WrongCaseMethod@6:4-6:12: Method name 'Greeter::SAYhello' has incorrect casing; use 'sayHello'
