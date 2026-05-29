===description===
reports too few method arguments
===file===
<?php
class Greeter {
    public function say(string $name, string $suffix): void {}
}
(new Greeter())->say('Ada');
===expect===
UnusedParam@3:25-3:37: Parameter $name is never used
UnusedParam@3:39-3:53: Parameter $suffix is never used
TooFewArguments@5:1-5:28: Too few arguments for say(): expected 2, got 1
