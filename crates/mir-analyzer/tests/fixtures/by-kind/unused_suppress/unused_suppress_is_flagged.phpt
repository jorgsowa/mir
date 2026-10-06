===description===
Unused suppress is flagged
===file===
<?php
class Foo {
    /**
     * @suppress UndefinedClass
//               ^^^^^^^^^^^^^^ UnusedSuppress: Suppress annotation for 'UndefinedClass' is never used
     */
    public string $bar = "baz";
}

===expect===
