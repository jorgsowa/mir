===description===
MissingReturnType fires for each interface method without a return type; a native
hint or a docblock @return suppresses it.
===file===
<?php
interface IFoo {
    public function noReturn();
    public function alsoNoReturn();
    public function hinted(): string;
    /** @return int */
    public function withDocblock();
}
===expect===
MissingReturnType@3:4-3:31: Function IFoo::noReturn() has no return type annotation
MissingReturnType@4:4-4:35: Function IFoo::alsoNoReturn() has no return type annotation
