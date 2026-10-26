===description===
Before PHP 8.5 a used trait's property is also compared with the one the class inherits;
a private parent property is not inherited, and an identical declaration is compatible.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Grand {
    public $label = 'a';
    protected $count = 0;
    private $secret = 'x';
    public $same = 1;
}
class Base extends Grand {}
trait Counter {
    public $label = 1;
    public $count = 0;
    public $secret = 1;
    public $same = 1;
}
class Child extends Base {
    use Counter;
//      ^^^^^^^^ CompileError: Grand and Counter define the same property ($label) in the composition of Child. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
//      ^^^^^^^^ CompileError: Grand and Counter define the same property ($count) in the composition of Child. However, the definition differs and is considered incompatible. Class was composed [trait-property-conflict]
}
