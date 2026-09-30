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
//      ^^^^^^^^^^^^^^^ UndefinedMethod: Method Base::secret() does not exist
    }
}
===expect===
