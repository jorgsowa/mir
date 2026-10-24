===description===
`\Foo\bar()` names `Foo\bar` even when `use Other\Foo;` imports a namespace
of the same name; only a qualified name goes through the class import.
===file===
<?php
namespace Foo {
    function bar(string $s): string { return $s; }
}

namespace Other\Foo {
    function bar(int $s): array { return [$s]; }
}

namespace App {
    use Other\Foo;

    function fullyQualified(): int {
        return \Foo\bar('a');
//      ^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'string' is not compatible with declared 'int'
    }

    function qualified(): int {
        return Foo\bar(1);
//      ^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'array' is not compatible with declared 'int'
    }
}
