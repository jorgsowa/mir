===description===
invalidUnionAssertion
===file===
<?php
interface I {
    /**
     * @assert null|!ExpectedType $value
     */
    public static function foo($value);
}
===expect===
InvalidDocblock
===ignore===
TODO
