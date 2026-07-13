===description===
calling an inaccessible static method dispatches to __callStatic when the class defines one, not UndefinedMethod
===file===
<?php
class WithMagic {
    private static function secret(): void {}
    public static function __callStatic($method, $parameters) { return null; }
}
WithMagic::secret();
===expect===
