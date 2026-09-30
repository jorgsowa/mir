===description===
reports too few method arguments
===file===
<?php
class Greeter {
    public function say(string $name, string $suffix): void {}
//                      ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
//                                    ^^^^^^^^^^^^^^ UnusedParam: Parameter $suffix is never used
}
(new Greeter())->say('Ada');
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for say(): expected 2, got 1
===expect===
