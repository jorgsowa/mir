===description===
`new self()` is reported as a class reference on the `self` keyword; `self` in a type position is not.
===cursor===
references
===file===
<?php
final class Factory {
    public static function make(): self { return new self(); }
}
Fact<CURSOR>ory::make();
===expect===
test.php@3:53-3:57
test.php@5:0-5:7
