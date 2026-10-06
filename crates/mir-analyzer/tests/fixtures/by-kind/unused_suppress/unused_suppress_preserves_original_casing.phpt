===description===
An unused named suppression's message quotes the author's original casing,
not a normalized form, even though matching itself is case-insensitive.
===file===
<?php
class Foo {
    /**
     * @suppress undefinedclass
//               ^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'undefinedclass' is never used
     */
    public string $bar = "baz";
}
===expect===
