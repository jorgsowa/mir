===file===
<?php
interface Fluent {
    public function retStatic(): static;
}

class A implements Fluent {
    public function foo(string $param): void { var_dump($param); }
    public function retStatic(): static { return $this; }
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
