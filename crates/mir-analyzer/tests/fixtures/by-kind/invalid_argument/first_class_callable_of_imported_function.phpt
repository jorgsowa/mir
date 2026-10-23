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
    }

    function fullyQualified(): void {
        $f = \Foo\bar(...);
        $f([]);
    }
}
===expect===
InvalidArgument@11:11-11:13: Argument $s of {closure}() expects 'string', got 'array{}'
InvalidArgument@16:11-16:13: Argument $s of {closure}() expects 'string', got 'array{}'
