===description===
reports too many method arguments
===file===
<?php
class Greeter {
    public function say(string $name): void {}
}
(new Greeter())->say('Ada', 'Grace');
===expect===
UnusedParam@3:24: Parameter $name is never used
TooManyArguments@5:28: Too many arguments for say(): expected 1, got 2
