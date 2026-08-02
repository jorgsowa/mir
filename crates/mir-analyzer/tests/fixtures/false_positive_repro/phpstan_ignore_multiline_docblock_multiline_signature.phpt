===description===
The compound shape: a multi-line `@phpstan-ignore` docblock above a
declaration whose OWN signature also spans multiple physical lines. The
directive must still land on the signature's first line, and per-parameter
issues on later continuation lines (the undefined type hint on `$x`) must
stay covered too.
===file===
<?php
class C {
    /**
     * @phpstan-ignore undefinedClass
     */
    private function bar(
        NoSuchClass $x,
        int $y
    ): void {
    }
}
===expect===
