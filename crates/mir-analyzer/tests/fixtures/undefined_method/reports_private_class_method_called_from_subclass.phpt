===file===
<?php
class Base {
    private function secret(): void {}
}
class Child extends Base {
    public function run(): void {
        $this->secret();
    }
}
===expect===
UndefinedMethod: Method Base::secret() does not exist
