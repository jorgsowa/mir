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
InvalidDocblock@3:0-3:0: Invalid docblock: @return contains variable `$thus` in type position
