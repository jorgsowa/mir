===file===
<?php
class A {
    public function foo(string $param): void { var_dump($param); }
    public function retStatic(): static { return $this; }
}

class B extends A {
    public function foo(array|string $param): void { var_dump($param); }
}

class C extends B {
    public function bar(): void {
        $array = [];
        $x = $this->retStatic();
        $x->foo($array);
    }
}
===expect===
