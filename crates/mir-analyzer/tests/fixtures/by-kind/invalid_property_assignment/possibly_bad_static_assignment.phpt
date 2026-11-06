===description===
Possibly bad static assignment
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    /** @var string */
    public static $foo = "a";

    public function barBar(): void
    {
        self::$foo = rand(0, 1) ? 5 : "hello";
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidPropertyAssignmentValue: Property $foo expects 'string', the assigned '5|"hello"' might not match
    }
}
