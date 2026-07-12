===description===
A private method called only from a trait's own method body ($this->helper()
inside the trait, where helper() is supplied by the composing class) must
not be reported unused.
===config===
suppress=
===file===
<?php
trait T {
    public function pub(): void {
        $this->helper();
    }
}

class Foo {
    use T;

    private function helper(): void {}
}

(new Foo())->pub();
===expect===
