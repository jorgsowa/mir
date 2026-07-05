===description===
@psalm-self-out also retypes `$this` when a method calls another self-out
method on itself.
===config===
suppress=UnusedParam
===file===
<?php
class MaybeString {
    /** @psalm-self-out ReadyString */
    public function withValue(string $v): void {}

    public function chain(): void {
        $this->withValue("x");
        /** @mir-check $this is ReadyString */
        $_ = 1;
    }
}
class ReadyString extends MaybeString {}

===expect===
