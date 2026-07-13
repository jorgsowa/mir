===description===
reports private static method called from outside
===file===
<?php
class Base {
    private static function secret(): void {}
}
Base::secret();
===expect===
UndefinedMethod@5:0-5:14: Method Base::secret() does not exist
