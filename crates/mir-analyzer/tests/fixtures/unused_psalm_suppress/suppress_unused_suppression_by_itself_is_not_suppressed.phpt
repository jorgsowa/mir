===description===
suppressUnusedSuppressionByItselfIsNotSuppressed
===file===
<?php
class Foo {
    /**
     * @suppress UnusedPsalmSuppress
     */
    public string $bar = "baz";
}

===expect===
UnusedPsalmSuppress
===ignore===
TODO
