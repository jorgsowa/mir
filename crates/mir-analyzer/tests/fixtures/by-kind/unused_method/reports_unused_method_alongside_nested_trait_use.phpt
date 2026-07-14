===description===
A truly-unused private method is still flagged even when the class also
composes a nested trait (`Outer` uses `Inner`) whose own methods reference an
unrelated private member — the transitive `traituse:` exemption walk must
not over-exempt.
===config===
suppress=
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
}

(new Foo())->pub();
===expect===
UnusedMethod@16:4-16:43: Private method Foo::trulyunused() is never called
