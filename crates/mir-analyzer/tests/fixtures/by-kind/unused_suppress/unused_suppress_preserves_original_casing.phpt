===description===
An unused named suppression's message quotes the author's original casing,
not a normalized form, even though matching itself is case-insensitive.
===file===
<?php
class Foo {
    /**
     * @suppress undefinedclass
     */
    public string $bar = "baz";
}
===expect===
UnusedSuppress@6:0-6:0: Suppress annotation for 'undefinedclass' is never used
