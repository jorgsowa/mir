===description===
reports private class method called from subclass
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
UndefinedMethod@7:9-7:24: Method Base::secret() does not exist
