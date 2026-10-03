===description===
No exception on missing class
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @suppress UndefinedClass */
class A
{
    /** @var class-string<Foo> */
    protected $bar;

    public function foo(string $s): void
    {
        $bar = $this->bar;
        $bar::baz();
//      ^^^^ UndefinedClass: Class Foo does not exist
    }
}
===expect===
UnusedSuppress@3:0-3:0: Suppress annotation for 'UndefinedClass' is never used
