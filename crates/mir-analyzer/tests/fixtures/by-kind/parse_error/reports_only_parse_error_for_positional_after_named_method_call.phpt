===description===
positional after named argument on method and static calls reports only the parse error
===file===
<?php
final class Greeter
{
    public function greet(string $name, int $times): void {}
    public static function shout(string $name, int $times): void {}
}
$g = new Greeter();
$g->greet(name: "a", 2);
//                   ^ ParseError: Parse error: cannot use positional argument after named argument
Greeter::shout(name: "a", 2);
//                        ^ ParseError: Parse error: cannot use positional argument after named argument
===expect===
