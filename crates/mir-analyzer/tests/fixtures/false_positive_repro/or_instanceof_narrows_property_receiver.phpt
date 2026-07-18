===description===
`$this->prop instanceof A || $this->prop instanceof B` must narrow the
property like its plain-variable counterpart already does.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
interface A {}
interface B {}

class Foo {
    public A|B|int $prop;

    // Positive: narrows away the `int` alternative.
    public function test(): void {
        if ($this->prop instanceof A || $this->prop instanceof B) {
            echo get_class($this->prop);
        }
    }
}

class Bar {
    public A|int $a;
    public B|int $b;

    // Negative: two different receivers must not be merged.
    public function test(A $other): void {
        if ($this->a instanceof A || $other instanceof B) {
            /** @mir-check $this->a is A|int */
            $x = $this->a;
            $_ = $x;
        }
    }
}
===expect===
