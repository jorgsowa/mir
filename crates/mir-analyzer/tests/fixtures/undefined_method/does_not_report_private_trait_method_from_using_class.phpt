===file===
<?php
trait MyTrait {
    private function privateMethod(): void {}
}
class MyClass {
    use MyTrait;
    public function run(): void {
        $this->privateMethod();
    }
}
===expect===
