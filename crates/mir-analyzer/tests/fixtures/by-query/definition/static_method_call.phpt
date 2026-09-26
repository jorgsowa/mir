===cursor===
definition
===file===
<?php
final class Factory {
    public static function make(): self { return new self(); }
}
Factory::ma<CURSOR>ke();
===expect===
test.php@3:4-3:62
