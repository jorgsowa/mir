===description===
Same gap as the closure sibling, but for an arrow function's own `@param`
docblock — a separate, duplicated code path
(analyze_arrow_function) that needed the identical fix. An arrow
function's body is a single expression, so the proof here is the
call-site RESULT type: `$r['ok']` on the resolved shape yields `bool`;
on the still-unresolved literal class atom `Result` it would not.
===config===
suppress=UnusedVariable,MissingClosureReturnType
===file===
<?php
/** @psalm-type Result = array{ok: bool, value: mixed} */
class Processor {
    public function run(): void {
        $fn = /** @param Result $r */
            fn($r) => $r['ok'];
        $result = $fn(['ok' => true, 'value' => 1]);
        /** @mir-check $result is bool */
        $_ = 1;
    }
}
===expect===
