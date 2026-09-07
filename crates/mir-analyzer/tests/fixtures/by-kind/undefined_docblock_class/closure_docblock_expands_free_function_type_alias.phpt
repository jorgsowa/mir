===description===
Same gap as the class-method sibling, but the alias table comes from the
enclosing FREE function's own docblock (`ctx.current_function_fqn`
branch) rather than a class-like's, when the closure is declared outside
any class body.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
/** @psalm-type Result = array{ok: bool, value: mixed} */
function process(): void {
    $fn = /** @param Result $r */
        function ($r) {
            /** @mir-check $r is array{ok: bool, value: mixed} */
            $_ = 1;
        };
}
===expect===
