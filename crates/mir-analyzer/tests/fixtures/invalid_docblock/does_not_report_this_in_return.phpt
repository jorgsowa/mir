===description===
does not report this in return
===file===
<?php
class Foo {
    /**
     * @return $this
     */
    public function self(): static { return $this; }
}
===expect===
===ignore===
TODO
