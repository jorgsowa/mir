===description===
Unused suppress is flagged
===file===
<?php
class Foo {
    /**
     * @suppress UndefinedClass
     */
    public string $bar = "baz";
}

===expect===
UnusedSuppress@6:0-6:0: Suppress annotation for 'UndefinedClass' is never used
