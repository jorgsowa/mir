===description===
The class token of a static call resolves to the class, separately from the method.
===cursor===
symbol
===file===
<?php
final class Factory {
    public static function make(): self { return new self(); }
}
Fact<CURSOR>ory::make();
===expect===
kind: class Factory
type: class-string
