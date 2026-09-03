===description===
A user-defined `@psalm-assert`/`@psalm-assert !Type` function narrows a
property-access argument, not just a bare variable — `apply_docblock_assertions`
only tried `extract_var_name`, unlike the built-in `assert()` path which
already handles properties. An unrelated property stays untouched.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
/** @psalm-assert string $value */
function assertIsStringProbe(mixed $value): void {}

/** @psalm-assert !null $x */
function assertNotNullProbe(mixed $x): void {}

class Holder {
    public mixed $prop;
    public mixed $other;

    public function testPlainAssertNarrowsProp(): void {
        assertIsStringProbe($this->prop);
        /** @mir-check $this->prop is string */
        echo $this->prop;
        /** @mir-check $this->other is mixed */
        $_ = 1;
    }

    public function testNegatedAssertNarrowsProp(?string $s): void {
        $this->prop = $s;
        assertNotNullProbe($this->prop);
        /** @mir-check $this->prop is string */
        echo $this->prop;
    }
}
===expect===
