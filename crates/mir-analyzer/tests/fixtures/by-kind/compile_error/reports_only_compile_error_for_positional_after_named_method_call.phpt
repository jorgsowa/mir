===description===
positional after named argument on method and static calls reports only the compile error
===file===
<?php
final class Greeter
{
    public function greet(string $name, int $times): string { return str_repeat($name, $times); }
    public static function shout(string $name, int $times): string { return str_repeat($name, $times); }
}
$g = new Greeter();
$g->greet(name: "a", 2);
//                   ^ CompileError: Compile error: Cannot use positional argument after named argument [positional-after-named-argument]
Greeter::shout(name: "a", 2);
//                        ^ CompileError: Compile error: Cannot use positional argument after named argument [positional-after-named-argument]
