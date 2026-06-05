===description===
Calling an instance method with correct casing is not reported.
===file===
<?php
class Greeter {
    public function sayHello(): void {}
}
$g = new Greeter();
$g->sayHello();
===expect===
