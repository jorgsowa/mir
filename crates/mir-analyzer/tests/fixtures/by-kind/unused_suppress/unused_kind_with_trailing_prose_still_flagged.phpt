===description===
Negative control: a suppressed kind followed by free-text prose must still
be flagged `UnusedSuppress` when genuinely unused — the prose must not
accidentally swallow the real kind name along with itself.
===file===
<?php
class Foo {
    /**
     * @suppress UndefinedClass because it's fine, actually not needed
     */
    public string $bar = "baz";
}
===expect===
UnusedSuppress@6:0-6:0: Suppress annotation for 'UndefinedClass' is never used
