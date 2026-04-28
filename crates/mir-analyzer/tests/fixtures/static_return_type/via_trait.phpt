===file===
<?php
trait RetStaticTrait {
    public function retStatic(): static { return $this; }
}

class A {
    use RetStaticTrait;
    public function foo(string $param): void { var_dump($param); }
}

class B extends A {
    public function foo(array|string $param): void { var_dump($param); }
}

class C extends B {
    public function bar(): void {
        $array = [];
        $this->retStatic()->foo($array);
    }
}
===expect===
