===file===
<?php
class Foo {
    /**
     * @return $this
     */
    public function self(): static { return $this; }
}
===expect===
