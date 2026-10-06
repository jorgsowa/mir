===description===
Negative control: a suppressed kind followed by free-text prose must still
be flagged `UnusedSuppress` when genuinely unused — the prose must not
accidentally swallow the real kind name along with itself.
===file===
<?php
class Foo {
    /**
     * @suppress UndefinedClass because it's fine, actually not needed
//               ^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UndefinedClass' is never used
     */
    public string $bar = "baz";
}
===expect===
