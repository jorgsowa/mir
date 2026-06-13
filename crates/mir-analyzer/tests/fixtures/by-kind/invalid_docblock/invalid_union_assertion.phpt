===description===
Invalid union assertion
===config===
suppress=MissingParamType,MissingReturnType
===file===
<?php
interface I {
    /**
     * @assert null|!ExpectedType $value
     */
    public static function foo($value);
}
===expect===
