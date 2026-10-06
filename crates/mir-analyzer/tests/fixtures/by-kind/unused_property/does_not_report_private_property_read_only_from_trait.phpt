===description===
A private property read only from a trait's own method body ($this->secret
inside the trait, where $secret is supplied by the composing class) must
not be reported unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
trait T {
    public function reveal(): string {
        return $this->secret;
//      ^^^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
    }
}

class Foo {
    use T;

    private string $secret = 'x';
}

echo (new Foo())->reveal();
