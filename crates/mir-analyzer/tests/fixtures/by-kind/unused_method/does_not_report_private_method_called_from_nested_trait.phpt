===description===
Same as `does_not_report_private_method_called_only_from_trait.phpt`, but the
call site lives in a trait composed transitively (`Outer` uses `Inner`,
`Foo` uses `Outer`) — the class's own direct trait list doesn't include
`Inner`, so the exemption must still reach it.
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
}

(new Foo())->pub();
===expect===
