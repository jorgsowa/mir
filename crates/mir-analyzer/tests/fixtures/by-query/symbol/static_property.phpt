===description===
A static property fetch resolves to the declaring class's property.
===cursor===
symbol
===file===
<?php
final class Config {
    public static string $env = 'prod';
}
echo Config::$e<CURSOR>nv;
===expect===
kind: property Config::$env
type: string
