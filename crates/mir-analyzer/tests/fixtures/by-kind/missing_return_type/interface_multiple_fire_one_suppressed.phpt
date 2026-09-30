===description===
MissingReturnType fires for each interface method without a return type; a native
hint or a docblock @return suppresses it.
===file===
<?php
interface IFoo {
    public function noReturn();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function IFoo::noReturn() has no return type annotation
    public function alsoNoReturn();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingReturnType: Function IFoo::alsoNoReturn() has no return type annotation
    public function hinted(): string;
    /** @return int */
    public function withDocblock();
}
===expect===
