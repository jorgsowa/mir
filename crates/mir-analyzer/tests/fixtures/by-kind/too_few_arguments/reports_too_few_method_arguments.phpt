===description===
reports too few method arguments
===file===
<?php
class Greeter {
    public function say(string $name, string $suffix): void {}
}
(new Greeter())->say('Ada');
===expect===
UnusedParam@3:24-3:36: Parameter $name is never used
UnusedParam@3:38-3:52: Parameter $suffix is never used
TooFewArguments@5:0-5:27: Too few arguments for say(): expected 2, got 1
