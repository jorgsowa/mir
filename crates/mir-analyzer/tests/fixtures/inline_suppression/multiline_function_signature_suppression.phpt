===description===
A suppression directive above a function declaration whose OWN signature
spans multiple physical lines (one parameter per line) previously only
covered the signature's first line — a per-parameter `UnusedParam`
diagnostic on a later parameter's own line escaped the directive entirely.
Contrast with an identical single-line signature, where suppression
already worked.
===file===
<?php
class Foo {
    // @mir-ignore
    private function bar(
        int $a,
        string $b
    ): void {
    }

    // @mir-ignore
    private function baz(int $c, string $d): void {
    }
}
===expect===
