===file:A.php===
<?php
class A {
    public function foo(string $param): void { var_dump($param); }
    public function retStatic(): static { return $this; }
}
===file:B.php===
<?php
class B extends A {
    public function foo(array|string $param): void { var_dump($param); }
}
===file:C.php===
<?php
class C extends B {
    public function bar(): void {
        $array = [];
        $this->foo($array);
        $this->retStatic()->foo($array);
    }
}
===expect===
