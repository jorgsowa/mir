===cursor===
symbol
===file===
<?php
final class Factory {
    public static function make(): self { return new self(); }
}
Factory::ma<CURSOR>ke();
===expect===
kind: static call Factory::make
type: Factory
