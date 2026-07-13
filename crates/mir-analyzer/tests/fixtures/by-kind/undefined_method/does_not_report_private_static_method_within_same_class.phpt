===description===
does not report private static method called via self:: within its own class
===file===
<?php
class Base {
    private static function secret(): void {}
    public static function run(): void {
        self::secret();
    }
}
===expect===
