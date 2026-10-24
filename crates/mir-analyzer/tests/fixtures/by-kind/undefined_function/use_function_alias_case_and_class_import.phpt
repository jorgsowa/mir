===description===
A `use function` alias matches case-insensitively, like PHP function names, and
a class import never names a function: `Tool()` with `use Lib\Sub\Tool;` is
`App\Tool()` or the global `Tool()`, not the function `Lib\Sub\Tool()`.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib {
    function Fmt(string $s): string { return $s; }
    function helper(): int { return 1; }
    class Tool {}
}
namespace Lib\Sub {
    function tool(): string { return 'x'; }
}
namespace App {
    use function Lib\Fmt;
    use Lib\Sub\Tool;

    function a(): int { return fmt('x'); }
//                      ^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'string' is not compatible with declared 'int'
    function b(): int { return Tool(); }
//                             ^^^^^^ UndefinedFunction: Function Tool() is not defined
}
