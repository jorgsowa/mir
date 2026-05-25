===description===
Invalid class method return
===file===
<?php
class C {
    /**
     * @return $thus
     */
    public function barBar() {
        return $this;
    }
}
===expect===
MissingDocblockType
===ignore===
TODO
