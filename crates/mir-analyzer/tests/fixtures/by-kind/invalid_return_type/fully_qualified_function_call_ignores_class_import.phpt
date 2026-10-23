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
    }

    function qualified(): int {
        return Foo\bar(1);
    }
}
===expect===
InvalidReturnType@14:8-14:29: Return type 'string' is not compatible with declared 'int'
InvalidReturnType@18:8-18:26: Return type 'array' is not compatible with declared 'int'
