===description===
Invalid union assertion
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {
    /**
     * @assert null|!ExpectedType $value
     */
    public static function foo($value);
}
