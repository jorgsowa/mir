===description===
Same as `does_not_report_private_property_read_only_from_trait.phpt`, but the
read site lives in a transitively-composed trait (`Outer` uses `Inner`, `Foo`
uses `Outer`).
===config===
suppress=
===file===
<?php
trait Inner {
    public function reveal(): string {
        return $this->secret;
    }
}

trait Outer {
    use Inner;
}

class Foo {
    use Outer;

    private string $secret = 'x';
}

echo (new Foo())->reveal();
===expect===
MixedReturnStatement@4:8-4:29: Cannot return a mixed type from function with declared return type 'string'
