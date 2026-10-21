===description===
Static properties, class constants and `::class` resolve fully qualified and relative names.
===file===
<?php
namespace App;

class A {
    public const C = 1;
    public static int $p = 1;
}

function ok(): int {
    $a = \App\A::$p + namespace\A::$p;
    $b = \App\A::C + namespace\A::C;
    echo \App\A::class, namespace\A::class;
    return $a + $b;
}

function bad(): void {
    echo \App\A::NOPE;
    echo namespace\A::NOPE;
}
===expect===
UndefinedConstant@17:9-17:21: Constant App\A::NOPE is not defined
UndefinedConstant@18:9-18:26: Constant App\A::NOPE is not defined
