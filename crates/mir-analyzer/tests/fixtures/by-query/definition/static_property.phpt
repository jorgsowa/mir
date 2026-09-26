===description===
Go-to-definition on a static property fetch lands on the property.
===cursor===
definition
===file===
<?php
final class Config {
    public static string $env = 'prod';
}
echo Config::$e<CURSOR>nv;
===expect===
test.php@3:4-3:38
