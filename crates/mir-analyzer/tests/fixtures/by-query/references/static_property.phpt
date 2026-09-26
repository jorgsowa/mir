===description===
Static property references include `self::` and external fetches.
===cursor===
references
===file===
<?php
final class Config {
    public static string $env = 'prod';
    public static function env(): string { return self::$env; }
}
echo Config::$e<CURSOR>nv;
Config::$env = 'dev';
===expect===
test.php@4:57-4:60
test.php@6:14-6:17
test.php@7:9-7:12
