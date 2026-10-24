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
//                      ^^^^^^^^^^^ InvalidReturnType: Return type 'string' is not compatible with declared 'int'
    function b(): string { return v; }
//                         ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'string'
    function c(): int { return LIMIT(); }
//                             ^^^^^^^ UndefinedFunction: Function LIMIT() is not defined
    function d(): string { return helper; }
//                                ^^^^^^ UndefinedConstant: Constant helper is not defined
}
