===description===
reports too many method arguments
===file===
<?php
class Greeter {
    public function say(string $name): void {}
//                      ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
}
(new Greeter())->say('Ada', 'Grace');
//                          ^^^^^^^ TooManyArguments: Too many arguments for say(): expected 1, got 2
