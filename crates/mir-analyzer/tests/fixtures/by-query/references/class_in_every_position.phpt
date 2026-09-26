===description===
Class references cover `new`, type hints, `instanceof`, `extends`, static calls, constants and `::class`.
===cursor===
references
===file===
<?php
class Greeter {
    public const NAME = 'g';
    public static function make(): Greeter { return new Greeter(); }
}
final class Loud extends Greeter {}
function f(Greeter $g): ?Greeter {
    if ($g instanceof Loud) { return new Greeter(); }
    echo Greeter::NAME, Greeter::class;
    return Gree<CURSOR>ter::make();
}
function g(object $o): bool { return $o instanceof Greeter; }
===expect===
test.php@4:35-4:42
test.php@4:56-4:63
test.php@6:25-6:32
test.php@7:11-7:18
test.php@7:25-7:32
test.php@8:41-8:48
test.php@9:9-9:16
test.php@9:24-9:31
test.php@10:11-10:18
test.php@12:51-12:58
