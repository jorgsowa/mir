===description===
A qualified constant name (`Foo\BAR`) resolves through a `use` import of its
first segment, else the current namespace, and never falls back to the global
`Foo\BAR` (only unqualified names do).
===file===
<?php
namespace Lib\Foo { const BAR = 1; }
namespace Foo { const BAZ = 2; }
namespace App {
    use Lib\Foo;
    function f(): void {
        echo Foo\BAR;
        echo Foo\BAZ;
//           ^^^^^^^ UndefinedConstant: Constant Lib\Foo\BAZ is not defined
    }
}
namespace Other {
    function g(): void {
        echo Foo\BAZ;
//           ^^^^^^^ UndefinedConstant: Constant Other\Foo\BAZ is not defined
        echo \Foo\BAZ;
    }
}
