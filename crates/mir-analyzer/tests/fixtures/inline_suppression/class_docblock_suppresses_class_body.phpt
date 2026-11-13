===description===
A `@psalm-suppress` on a class-like's docblock covers its whole body, as Psalm does,
for the named kinds only. A following class is not covered.
===file===
<?php
/** @psalm-suppress InvalidReturnType */
final class P {
    public static function f(): int { return "x"; }

    public function g(): int {
        echo 1;
        return "y";
    }

    public function other(?P $p): void {
        $p->g();
//      ^^^^^^^ PossiblyNullMethodCall: Cannot call method g() on possibly null value
    }
}

/**
 * @psalm-suppress UndefinedClass
 */
abstract class Q {
    public function make(): object { return new Missing(); }
}

/** @psalm-suppress InvalidReturnType */
trait T {
    public function t(): int { return "t"; }
}

final class R {
    public function h(): int { return "z"; }
//                             ^^^^^^^^^^^ InvalidReturnType: Return type '"z"' is not compatible with declared 'int'
}
