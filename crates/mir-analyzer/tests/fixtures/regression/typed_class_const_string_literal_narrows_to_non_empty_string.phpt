===description===
Same root cause as the typed-int-const case, on the `string` sibling: a native `string`
hint on a class constant discarded literal inference, so a non-empty literal never
narrowed to `non-empty-string`.
===config===
suppress=UnusedParam
===file===
<?php
final class Foo {
    private const string NAME = 'widget';
    public function bar(): void {
        baz([self::NAME]);
    }
}

/** @param list<non-empty-string> $names */
function baz(array $names): void {}

===expect===
