===description===
reports too many static method arguments
===file===
<?php
class Greeter {
    public static function say(string $name): void {}
}
Greeter::say('Ada', 'Grace');
===expect===
UnusedParam@3:32-3:44: Parameter $name is never used
TooManyArguments@5:21-5:28: Too many arguments for say(): expected 1, got 2
