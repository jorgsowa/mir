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
InvalidDocblock@4:7-4:20: Invalid docblock: @return contains variable `$thus` in type position
