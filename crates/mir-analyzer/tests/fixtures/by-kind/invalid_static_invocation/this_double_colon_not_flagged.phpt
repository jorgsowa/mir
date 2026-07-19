===description===
`$this::method()` is self-referential (LSB against the current instance),
same as self::/static::/parent:: — calling a non-static method through it
must not fire InvalidStaticInvocation, and @psalm-self-out narrowing must
still apply. An unrelated object-variable receiver (`$g::`) stays flagged.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Greeter {
    public function hello(): string { return "hello"; }

    public function callViaThisDoubleColon(): string {
        return $this::hello();
    }
}

// Negative: a non-$this object-variable receiver is unaffected.
function callViaObjectVariable(Greeter $g): string {
    return $g::hello();
}

class MaybeString {
    /** @psalm-self-out ReadyString */
    public function withValue(string $v): void {}

    public function chainViaThisDoubleColon(): void {
        $this::withValue("x");
        /** @mir-check $this is ReadyString */
        $_ = 1;
    }
}
class ReadyString extends MaybeString {}
===expect===
InvalidStaticInvocation@12:11-12:22: Non-static method Greeter::hello() cannot be called statically
