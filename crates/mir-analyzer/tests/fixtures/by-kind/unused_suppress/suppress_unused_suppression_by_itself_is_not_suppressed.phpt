===description===
Suppress unused suppression by itself is not suppressed
===file===
<?php
class Foo {
    /**
     * @suppress UnusedSuppress
     */
    public string $bar = "baz";
}

===expect===
