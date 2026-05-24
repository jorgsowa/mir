===description===
reports too few method arguments
===file===
<?php
class Greeter {
    public function say(string $name, string $suffix): void {}
}
(new Greeter())->say('Ada');
===expect===
UnusedParam@3:25: Parameter $name is never used
UnusedParam@3:39: Parameter $suffix is never used
TooFewArguments@5:1: Too few arguments for say(): expected 2, got 1
