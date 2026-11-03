===description===
Non-final, abstract, interface, trait and enum declarations nothing references are reported
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class PlainUnused {}
//<^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class PlainUnused is never referenced
abstract class AbstractUnused {}
//       ^^^^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class AbstractUnused is never referenced
interface InterfaceUnused {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class InterfaceUnused is never referenced
trait TraitUnused {}
//<^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class TraitUnused is never referenced
enum EnumUnused { case A; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedClass: Class EnumUnused is never referenced

abstract class Base {}
interface Contract {}
trait Helpers {}
final class Child extends Base implements Contract {
    use Helpers;
}

new Child();
