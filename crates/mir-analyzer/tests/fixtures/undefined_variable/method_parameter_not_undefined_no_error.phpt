===description===
method parameter not undefined no error
===file===
<?php
class Greeter {
    public function greet(string $name): string {
        return 'Hello, ' . $name;
    }
}
===expect===
===ignore===
TODO
