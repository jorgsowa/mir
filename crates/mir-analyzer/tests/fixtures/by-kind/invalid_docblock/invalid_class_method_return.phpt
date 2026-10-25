===description===
Invalid class method return
===file===
<?php
class C {
    /**
     * @return $thus
//     ^^^^^^^^^^^^^ InvalidDocblock: @return contains variable `$thus` in type position
     */
    public function barBar() {
        return $this;
    }
}
