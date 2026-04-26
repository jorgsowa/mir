===file===
<?php
class Greeter {
    public function say(string $name, string $suffix): void {}
}
(new Greeter())->say('Ada');
===expect===
UnusedParam: Parameter $name is never used
UnusedParam: Parameter $suffix is never used
TooFewArguments: Too few arguments for say(): expected 2, got 1
