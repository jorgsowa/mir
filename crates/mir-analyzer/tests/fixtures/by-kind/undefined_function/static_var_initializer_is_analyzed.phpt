===description===
FN: `static $x = <expr>;` never analyzed its initializer expression, so an
undefined-function call inside it went uncaught and the variable's type
was unconditionally `mixed` regardless of the initializer's real type.
===file===
<?php
function foo(): void {
    static $x = totallyUndefinedFunctionXyz();
    echo $x;
}
===expect===
UndefinedFunction@3:16-3:45: Function totallyUndefinedFunctionXyz() is not defined
