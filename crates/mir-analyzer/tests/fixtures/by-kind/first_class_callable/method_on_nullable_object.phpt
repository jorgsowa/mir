===description===
P3: First-class callable from an instance method on a nullable object still produces
a typed closure (the object itself is non-null at the call site, `$obj?->m(...)` is
a different syntax).
===config===
suppress=UnusedVariable
===file===
<?php

class Parser {
    public function parse(string $input): int { return (int) $input; }
}

/** @param Parser $p */
function makeParser(Parser $p): \Closure {
    return $p->parse(...);
}
===expect===
