===description===
InvalidStaticInvocation fires when an instance method is called statically.
===file===
<?php
class Greeter {
    public function hello(): string { return "hello"; }
}

Greeter::hello();
//<^^^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Greeter::hello() cannot be called statically
===expect===
