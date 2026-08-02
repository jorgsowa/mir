===description===
A `try`/`catch` where the only catch clause diverges (rethrows, wrapped in a
new exception) reading a function parameter nowhere else in the function.
`analyze_trycatch_stmt` folded only non-diverging catch clauses into the
post-statement flow state, silently dropping a diverging clause's
`read_vars` entirely — so the param's only read, from inside the diverging
catch, never counted as a use. Modeled on doctrine/instantiator's
`try { ... } catch (Throwable $e) { throw new Exception($reflectionClass->getName()); }`
idiom.
===config===
suppress=MissingThrowsDocblock
===file===
<?php
function f(\ReflectionClass $reflectionClass): void {
    try {
        maybeThrow();
    } catch (\Throwable $e) {
        throw new \Exception($reflectionClass->getName());
    }
}

function maybeThrow(): void {}
===expect===
