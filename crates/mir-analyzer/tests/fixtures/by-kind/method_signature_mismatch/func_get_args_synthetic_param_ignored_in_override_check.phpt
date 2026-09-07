===description===
FP-L27: a parent method calling `func_get_args()` gets a synthetic `...`
param injected onto its OWN declared-param list, purely so a call site can
pass more positional args than shown without a false TooManyArguments. That
synthetic entry isn't part of the real, caller-visible signature, but the
override "fewer parameters than parent" check counted it anyway — a child
overriding with the SAME real param list was wrongly flagged as having
fewer parameters than the (artificially inflated) parent.
===config===
suppress=UnusedParam
===file===
<?php
class A {
    public function fooFoo(int $a, bool $b): void {
        func_get_args();
    }
}

class B extends A {
    public function fooFoo(int $a, bool $b): void {
    }
}
===expect===
