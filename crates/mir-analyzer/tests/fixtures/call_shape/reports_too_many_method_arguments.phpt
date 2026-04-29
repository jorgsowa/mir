===file===
<?php
class Greeter {
    public function say(string $name): void {}
}
(new Greeter())->say('Ada', 'Grace');
===expect===
UnusedParam: Parameter $name is never used
TooManyArguments: Too many arguments for say(): expected 1, got 2
