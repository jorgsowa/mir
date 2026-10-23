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
    function b(): int { return MODE; }
    function c(): int { return limit; }
}
===expect===
InvalidReturnType@13:24-13:37: Return type '"x"' is not compatible with declared 'int'
UndefinedConstant@14:31-14:35: Constant MODE is not defined
UndefinedConstant@15:31-15:36: Constant limit is not defined
