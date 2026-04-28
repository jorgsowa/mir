===file===
<?php
class A {
    public function foo(string $param): void { var_dump($param); }
    public function retStatic(): static { return $this; }
}

class B extends A {
    public function foo(array|string $param): void {
        if (is_array($param)) $param = json_encode($param);
        parent::foo($param);
    }
}

class C extends B {
    public function bar(): void {
        $array = [];
        $this->foo($array);
        $this->retStatic()->foo($array);
    }
}
===expect===
PossiblyInvalidArgument: Argument $param of foo() expects 'string', possibly different type 'string|false' provided
