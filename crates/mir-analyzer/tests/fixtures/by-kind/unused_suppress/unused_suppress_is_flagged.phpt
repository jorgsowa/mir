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
UnusedSuppress@4:17-4:31: Suppress annotation for 'UndefinedClass' is never used
