===description===
InvalidStaticInvocation fires when an instance method is called statically.
===file===
<?php
class Greeter {
    public function hello(): string { return "hello"; }
}

Greeter::hello();
===expect===
InvalidStaticInvocation@6:1-6:17: Non-static method Greeter::hello() cannot be called statically
