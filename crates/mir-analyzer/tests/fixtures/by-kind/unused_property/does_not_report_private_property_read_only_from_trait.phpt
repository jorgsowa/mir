===description===
A private property read only from a trait's own method body ($this->secret
inside the trait, where $secret is supplied by the composing class) must
not be reported unused.
===config===
suppress=
===file===
<?php
trait T {
    public function reveal(): string {
        return $this->secret;
    }
}

class Foo {
    use T;

    private string $secret = 'x';
}

echo (new Foo())->reveal();
===expect===
MixedReturnStatement@4:8-4:29: Cannot return a mixed type from function with declared return type 'string'
