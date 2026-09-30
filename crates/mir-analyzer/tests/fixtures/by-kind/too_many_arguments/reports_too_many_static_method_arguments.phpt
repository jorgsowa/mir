===description===
reports too many static method arguments
===file===
<?php
class Greeter {
    public static function say(string $name): void {}
//                             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
}
Greeter::say('Ada', 'Grace');
//                  ^^^^^^^ TooManyArguments: Too many arguments for say(): expected 1, got 2
===expect===
