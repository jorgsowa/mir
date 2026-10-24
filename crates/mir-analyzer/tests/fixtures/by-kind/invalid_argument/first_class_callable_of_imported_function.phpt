===description===
A first-class callable resolves its name like a call: `use function` imports
and fully qualified names give the function's signature.
===file===
<?php
namespace Foo {
    function bar(string $s): string { return $s; }
}

namespace App {
    use function Foo\bar;

    function imported(): void {
        $f = bar(...);
        $f([]);
//         ^^ InvalidArgument: Argument $s of {closure}() expects 'string', got 'array{}'
    }

    function fullyQualified(): void {
        $f = \Foo\bar(...);
        $f([]);
//         ^^ InvalidArgument: Argument $s of {closure}() expects 'string', got 'array{}'
    }
}
