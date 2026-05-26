===description===
Dont report implementer error on abstract trait method twice
===file===
<?php
trait B {
    abstract public function run();
}

final class A {
    use B;

    #[Override]
    public function run(string $foo): string {
        return $foo;
    }
}
===expect===
MethodSignatureMismatch
===ignore===
TODO
