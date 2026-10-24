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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method A::run() signature mismatch: overriding method requires 1 argument(s) but parent requires 0
        return $foo;
    }
}
