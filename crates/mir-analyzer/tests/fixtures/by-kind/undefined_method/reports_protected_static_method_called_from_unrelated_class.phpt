===description===
reports protected static method called from an unrelated class
===file===
<?php
class Base {
    protected static function secret(): void {}
}
class Unrelated {
    public function run(): void {
        Base::secret();
    }
}
===expect===
UndefinedMethod@7:8-7:22: Method Base::secret() does not exist
