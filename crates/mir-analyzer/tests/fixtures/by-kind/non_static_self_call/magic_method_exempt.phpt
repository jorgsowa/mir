===description===
NonStaticSelfCall does NOT fire for methods whose names start with __ (magic methods are exempt).
===file===
<?php
class Service {
    public function __clone() {}

    public static function duplicate(): void {
        self::__clone();
    }
}
===expect===
