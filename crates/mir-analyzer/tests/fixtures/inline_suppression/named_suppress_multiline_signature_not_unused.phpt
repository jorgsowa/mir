===description===
A named suppression (`@psalm-suppress UnusedParam`) above a function whose
own signature spans multiple physical lines only recorded the signature's
FIRST line as its target for `UnusedSuppress` detection — the actual
`UnusedParam` issue fires on the unused parameter's OWN (later,
continuation) line, so the directive was misreported as unused even
though it genuinely suppressed a real issue.
===file===
<?php
class Foo {
    /**
     * @psalm-suppress UnusedParam
     */
    private function bar(
        int $used,
        int $notUsed
    ): void {
        echo $used;
    }
}
===expect===
