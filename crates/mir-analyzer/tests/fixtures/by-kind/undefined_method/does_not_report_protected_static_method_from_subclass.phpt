===description===
does not report protected static method called via parent:: from a subclass
===file===
<?php
class Base {
    protected static function secret(): void {}
}
class Child extends Base {
    public static function run(): void {
        parent::secret();
    }
}
===expect===
