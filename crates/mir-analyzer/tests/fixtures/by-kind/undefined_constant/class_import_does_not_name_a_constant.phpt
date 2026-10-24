===description===
`use const` imports a constant, but a class import of the same short name does
not: `MODE` with `use Lib\Sub\MODE;` is `App\MODE` or the global `MODE`.
Constant aliases are case-sensitive.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    const LIMIT = 'x';
    class LIMIT {}
}
namespace Lib\Sub {
    const MODE = 'y';
}
namespace App {
    use const Lib\LIMIT;
    use Lib\Sub\MODE;

    function a(): int { return LIMIT; }
//                      ^^^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
    function b(): int { return MODE; }
//                             ^^^^ UndefinedConstant: Constant MODE is not defined
    function c(): int { return limit; }
//                             ^^^^^ UndefinedConstant: Constant limit is not defined
}
