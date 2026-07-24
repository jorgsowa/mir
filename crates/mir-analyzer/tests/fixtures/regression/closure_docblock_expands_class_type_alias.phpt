===description===
A closure's own `@param` docblock never consulted the enclosing class's
local `@psalm-type` alias table — `Result` stayed the literal unresolved
atom instead of expanding to the shape, unlike a bare `@var` annotation
(stmt/mod.rs::extract_var_annotation_from), which already does this.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
/** @psalm-type Result = array{ok: bool, value: mixed} */
class Processor {
    public function run(): void {
        $fn = /** @param Result $r */
            function ($r) {
                /** @mir-check $r is array{ok: bool, value: mixed} */
                $_ = 1;
            };
    }
}
===expect===
