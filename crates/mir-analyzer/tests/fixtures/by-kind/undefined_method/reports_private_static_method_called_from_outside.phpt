===description===
reports private static method called from outside
===file===
<?php
class Base {
    private static function secret(): void {}
}
Base::secret();
//<^^^^^^^^^^^^^^ UndefinedMethod: Method Base::secret() does not exist
