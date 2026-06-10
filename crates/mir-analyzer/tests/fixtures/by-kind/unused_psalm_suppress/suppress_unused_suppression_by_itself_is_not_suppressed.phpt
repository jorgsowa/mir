===description===
Suppress unused suppression by itself is not suppressed
===file===
<?php
class Foo {
    /**
     * @suppress UnusedPsalmSuppress
     */
    public string $bar = "baz";
}

===expect===
