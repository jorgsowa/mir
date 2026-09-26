===description===
A constant inherited from an interface lands on the interface's declaration.
===cursor===
definition
===file===
<?php
interface HasVersion {
    public const VERSION = 2;
}
final class Api implements HasVersion {}
echo Api::VER<CURSOR>SION;
===expect===
test.php@3:4-3:29
