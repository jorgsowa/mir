===description===
FP-L4: a method body that unconditionally throws (no return statement,
never falls off the end) was inferred as `void` instead of `never` — the
bottom type, which satisfies any expected argument type at the call site.
An overrider that DOES return a real value is unaffected (own inferred
type wins there).
===config===
suppress=UnusedParam,MissingParamType
===file===
<?php

class Grammar {
    public function make($n) {
        throw new \LogicException('unsupported');
    }
}

function run(string $sql): void {}

function use1(Grammar $g, $n): void {
    run($g->make($n));
}
===expect===
