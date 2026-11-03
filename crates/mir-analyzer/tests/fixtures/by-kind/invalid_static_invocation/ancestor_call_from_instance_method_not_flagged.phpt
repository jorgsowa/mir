===description===
`Ancestor::method()` from an instance method of a subclass forwards `$this`, like `parent::`; from a static method it still fires.
===file===
<?php
abstract class Dumper {
    protected function dumpLine(int $depth): void { echo $depth; }
}
class CliDumper extends Dumper {}
class HtmlDumper extends CliDumper {
    protected function dumpLine(int $depth): void {
        Dumper::dumpLine($depth);
    }

    public static function make(): void {
        Dumper::dumpLine(0);
//      ^^^^^^^^^^^^^^^^^^^ InvalidStaticInvocation: Non-static method Dumper::dumpLine() cannot be called statically
    }
}
