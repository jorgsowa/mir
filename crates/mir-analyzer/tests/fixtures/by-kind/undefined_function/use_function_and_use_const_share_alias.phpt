===description===
`use function` and `use const` aliases live apart, as in PHP: both can import
`v`, and a constant import never names a function (`LIMIT()`) nor a function
import a constant (`helper`).
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Fns {
    function v(): string { return 's'; }
    function helper(): string { return 'h'; }
}
namespace Consts {
    const v = 1;
    const LIMIT = 2;
}
namespace App {
    use function Fns\v;
    use function Fns\helper;
    use const Consts\v;
    use const Consts\LIMIT;

    function a(): int { return v(); }
    function b(): string { return v; }
    function c(): int { return LIMIT(); }
    function d(): string { return helper; }
}
===expect===
InvalidReturnType@16:24-16:35: Return type 'string' is not compatible with declared 'int'
InvalidReturnType@17:27-17:36: Return type '1' is not compatible with declared 'string'
UndefinedFunction@18:31-18:38: Function LIMIT() is not defined
UndefinedConstant@19:34-19:40: Constant helper is not defined
