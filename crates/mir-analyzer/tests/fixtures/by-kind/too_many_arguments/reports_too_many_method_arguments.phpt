===description===
reports too many method arguments
===file===
<?php
class Greeter {
    public function say(string $name): void {}
}
(new Greeter())->say('Ada', 'Grace');
===expect===
UnusedParam@3:25-3:37: Parameter $name is never used
TooManyArguments@5:29-5:36: Too many arguments for say(): expected 1, got 2
