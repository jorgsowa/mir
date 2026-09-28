===description===
`self`/`static`/`parent` parse to their sentinel atoms inside a class body
without producing an invalid-docblock-type error.
===config===
suppress=MissingReturnType,MissingParamType,ForbiddenCode
===file===
<?php
class Base {
    public function checkSelf($this_var) {
        /**
         * @var self $this_var
         * @mir-check $this_var is self
         */
        var_dump($this_var);
    }

    public function checkStatic($this_var) {
        /**
         * @var static $this_var
         * @mir-check $this_var is static
         */
        var_dump($this_var);
    }
}

class Derived extends Base {
    public function checkParent($this_var) {
        /**
         * @var parent $this_var
         * @mir-check $this_var is parent
         */
        var_dump($this_var);
    }
}
===expect===
