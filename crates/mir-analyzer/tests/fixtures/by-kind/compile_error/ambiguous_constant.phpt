===description===
A constant inherited from two class-likes that declare it separately is ambiguous, whatever the values, unless the class declares its own. One declaration reached twice (an interface and its parent) is fine.
===config===
<mir>
  <phpVersion>8.5</phpVersion>
</mir>
===file===
<?php
interface First { const LIMIT = 1; }
interface Second { const LIMIT = 1; }
interface Child extends First {}
class Base { const LIMIT = 2; }

class BothInterfaces implements First, Second {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class BothInterfaces inherits both First::LIMIT and Second::LIMIT, which is ambiguous [ambiguous-constant]

class ParentAndInterface extends Base implements First {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Class ParentAndInterface inherits both Base::LIMIT and First::LIMIT, which is ambiguous [ambiguous-constant]

interface Extended extends First, Second {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Interface Extended inherits both First::LIMIT and Second::LIMIT, which is ambiguous [ambiguous-constant]

enum Mode implements First, Second { case A; }
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Enum Mode inherits both First::LIMIT and Second::LIMIT, which is ambiguous [ambiguous-constant]

class Redeclares implements First, Second { const LIMIT = 3; }

class Diamond implements First, Child {}
