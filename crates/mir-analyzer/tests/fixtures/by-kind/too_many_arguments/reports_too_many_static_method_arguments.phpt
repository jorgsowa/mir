===description===
reports too many static method arguments
===file===
<?php
class Greeter {
    public static function say(string $name): void {}
}
Greeter::say('Ada', 'Grace');
===expect===
UnusedParam@3:31-3:43: Parameter $name is never used
TooManyArguments@5:20-5:27: Too many arguments for say(): expected 1, got 2
