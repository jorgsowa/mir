===description===
A truly-unused private method is still flagged even when the class also
composes a nested trait (`Outer` uses `Inner`) whose own methods reference an
unrelated private member — the transitive `traituse:` exemption walk must
not over-exempt.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
trait Inner {
    public function pub(): void {
        $this->helper();
    }
}

trait Outer {
    use Inner;
}

class Foo {
    use Outer;

    private function helper(): void {}
    private function trulyunused(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedMethod: Private method Foo::trulyunused() is never called
}

(new Foo())->pub();
===expect===
