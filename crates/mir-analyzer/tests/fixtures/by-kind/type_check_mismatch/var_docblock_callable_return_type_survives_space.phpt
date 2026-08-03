===description===
M4: `@var callable(...): R` / `@var Closure(...): R` resolves its return
type even when a space follows the closing paren's `:` — both the
property form (nameless `@var`) and the named local-var form. The
no-space `callable(int):string` spelling already worked and must keep
working; a plain `@var string some description` (no callable signature at
all) must still stop at the type and not swallow the description.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
class Box {
    /** @var callable(int, array): string */
    public $fn;
    public function run(int $a, array $b): void {
        $result = ($this->fn)($a, $b);
        /** @mir-check $result is string */
        $_ = 1;
    }
}

function localVarForm(): void {
    /** @var callable(int): string $fn */
    $fn = strval(...);
    $result = $fn(3);
    /** @mir-check $result is string */
    $_ = 1;
}

function noSpaceStillWorks(): void {
    /** @var callable(int):string $fn */
    $fn = strval(...);
    $result = $fn(3);
    /** @mir-check $result is string */
    $_ = 1;
}

class Plain {
    /** @var string some description here */
    public $bar;
}
function descriptionNotSwallowed(Plain $p): void {
    /** @mir-check $p->bar is string */
    $_ = 1;
}
===expect===
