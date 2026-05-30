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
MethodSignatureMismatch@9:4-9:15: Method A::run() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
